//! This crate adds a middleware for [`actix-web`](https://actix.rs/) that captures errors and
//! report them to `Sentry`.
//!
//! To use this middleware just configure Sentry and then add it to your actix web app as a
//! middleware.  Because actix is generally working with non sendable objects and highly concurrent
//! this middleware creates a new hub per request.  As a result many of the sentry integrations
//! such as breadcrumbs do not work unless you bind the actix hub.
//!
//! # Example
//!
//! ```ignore
//! use std::env;
//! use std::io;
//!
//! use actix_web::{get, App, Error, HttpRequest, HttpServer};
//! use sentry::Level;
//!
//! #[get("/")]
//! async fn failing(_req: HttpRequest) -> Result<String, Error> {
//!     Err(io::Error::new(io::ErrorKind::Other, "An error happens here").into())
//! }
//!
//! #[actix_web::main]
//! async fn main() -> io::Result<()> {
//!     let _guard = sentry::init(());
//!     env::set_var("RUST_BACKTRACE", "1");
//!
//!     HttpServer::new(|| {
//!         App::new()
//!             .wrap(sentry_actix::Sentry::new())
//!             .service(failing)
//!     })
//!     .bind("127.0.0.1:3001")?
//!     .run()
//!     .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Reusing the Hub
//!
//! This integration will automatically update the current Hub instance. For example,
//! the following will capture a message in the current request's Hub:
//!
//! ```ignore
//! # fn test(req: &actix_web::HttpRequest) {
//! use sentry::Level;
//! sentry::capture_message("Something is not well", Level::Warning);
//! # }
//! ```

#![doc(html_favicon_url = "https://sentry-brand.storage.googleapis.com/favicon.ico")]
#![doc(html_logo_url = "https://sentry-brand.storage.googleapis.com/sentry-glyph-black.png")]
#![warn(missing_docs)]
#![allow(clippy::needless_doctest_main)]
#![allow(deprecated)]
#![allow(clippy::type_complexity)]

use std::{borrow::Cow, pin::Pin, sync::Arc};

use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::{
    future::{ok, Future, Ready},
    FutureExt,
};
use sentry_core::{
    protocol::{ClientSdkPackage, Event, Request},
    types::Uuid,
    Hub, SentryFutureExt,
};

#[cfg(feature = "eyre")]
pub mod eyre;

/// A helper construct that can be used to reconfigure and build the middleware.
pub struct SentryBuilder {
    middleware: Sentry,
}

impl SentryBuilder {
    /// Finishes the building and returns a middleware
    pub fn finish(self) -> Sentry {
        self.middleware
    }

    /// Reconfigures the middleware so that it uses a specific hub instead of the default one.
    pub fn with_hub(mut self, hub: Arc<Hub>) -> Self {
        self.middleware.hub = Some(hub);
        self
    }

    /// Reconfigures the middleware so that it uses a specific hub instead of the default one.
    pub fn with_default_hub(mut self) -> Self {
        self.middleware.hub = None;
        self
    }

    /// If configured the sentry id is attached to a X-Sentry-Event header.
    pub fn emit_header(mut self, val: bool) -> Self {
        self.middleware.emit_header = val;
        self
    }

    /// Enables or disables error reporting.
    ///
    /// The default is to report all errors.
    pub fn capture_server_errors(mut self, val: bool) -> Self {
        self.middleware.capture_server_errors = val;
        self
    }

    /// Enables or disables error reporting for client errors.
    ///
    /// The default is to report all errors.
    pub fn capture_client_errors(mut self, val: bool) -> Self {
        self.middleware.capture_client_errors = val;
        self
    }
}

/// Reports certain failures to Sentry.
#[derive(Clone)]
pub struct Sentry {
    hub: Option<Arc<Hub>>,
    emit_header: bool,
    capture_client_errors: bool,
    capture_server_errors: bool,
}

impl Sentry {
    /// Creates a new sentry middleware.
    pub fn new() -> Self {
        Sentry {
            hub: None,
            emit_header: false,
            capture_server_errors: true,
            capture_client_errors: false,
        }
    }

    /// Creates a new middleware builder.
    pub fn builder() -> SentryBuilder {
        Sentry::new().into_builder()
    }

    /// Converts the middleware into a builder.
    pub fn into_builder(self) -> SentryBuilder {
        SentryBuilder { middleware: self }
    }
}

impl Default for Sentry {
    fn default() -> Self {
        Sentry::new()
    }
}

impl<S, B> Transform<S> for Sentry
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Transform = SentryMiddleware<S>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(SentryMiddleware {
            service,
            inner: self.clone(),
        })
    }
}

/// The middleware for individual services.
pub struct SentryMiddleware<S> {
    service: S,
    inner: Sentry,
}

impl<S, B> Service for SentryMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let inner = self.inner.clone();
        let hub = Arc::new(Hub::new_from_top(
            inner.hub.clone().unwrap_or_else(Hub::main),
        ));
        let client = hub.client();
        let with_pii = client
            .as_ref()
            .map_or(false, |x| x.options().send_default_pii);

        let (tx, sentry_req) = sentry_request_from_http(&req, with_pii);
        hub.configure_scope(|scope| {
            scope.set_transaction(tx.as_deref());
            scope.add_event_processor(Box::new(move |event| process_event(event, &sentry_req)))
        });

        let fut = self.service.call(req).bind_hub(hub.clone());

        async move {
            // Service errors
            let mut res: Self::Response = match fut.await {
                Ok(res) => res,
                Err(e) => {
                    if inner.capture_server_errors || inner.capture_client_errors {
                        process_error(hub, &e);
                    }
                    return Err(e);
                }
            };

            // Response errors
            let status = res.response().status();
            if inner.capture_server_errors && status.is_server_error()
                || inner.capture_client_errors && status.is_client_error()
            {
                if let Some(e) = res.response().error() {
                    let event_id = process_error(hub, e);

                    if inner.emit_header {
                        res.response_mut().headers_mut().insert(
                            "x-sentry-event".parse().unwrap(),
                            event_id.to_simple_ref().to_string().parse().unwrap(),
                        );
                    }
                }
            }

            Ok(res)
        }
        .boxed_local()
    }
}

fn process_error(hub: Arc<Hub>, e: &actix_web::Error) -> Uuid {
    process_eyre_report(hub.clone(), e).unwrap_or_else(|| hub.capture_error(e))
}

#[cfg(feature = "eyre")]
fn process_eyre_report(hub: Arc<Hub>, e: &actix_web::Error) -> Option<Uuid> {
    use sentry_eyre::EyreHubExt;

    use self::eyre::WrapEyre;

    e.as_error::<WrapEyre>()
        .map(|report| hub.capture_eyre(report))
}

#[cfg(not(feature = "eyre"))]
fn process_eyre_report(_hub: Arc<Hub>, _e: &actix_web::Error) -> Option<Uuid> {
    None
}

/// Build a Sentry request struct from the HTTP request
fn sentry_request_from_http(request: &ServiceRequest, with_pii: bool) -> (Option<String>, Request) {
    let transaction = if let Some(name) = request.match_name() {
        Some(String::from(name))
    } else if let Some(pattern) = request.match_pattern() {
        Some(pattern)
    } else {
        None
    };

    let mut sentry_req = Request {
        url: format!(
            "{}://{}{}",
            request.connection_info().scheme(),
            request.connection_info().host(),
            request.uri()
        )
        .parse()
        .ok(),
        method: Some(request.method().to_string()),
        headers: request
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect(),
        ..Default::default()
    };

    // If PII is enabled, include the remote address
    if with_pii {
        if let Some(remote) = request.connection_info().remote_addr() {
            sentry_req.env.insert("REMOTE_ADDR".into(), remote.into());
        }
    };

    (transaction, sentry_req)
}

/// Add request data to a Sentry event
#[allow(clippy::unnecessary_wraps)]
fn process_event(mut event: Event<'static>, request: &Request) -> Option<Event<'static>> {
    // Request
    if event.request.is_none() {
        event.request = Some(request.clone());
    }

    // SDK
    if let Some(sdk) = event.sdk.take() {
        let mut sdk = sdk.into_owned();
        sdk.packages.push(ClientSdkPackage {
            name: "sentry-actix".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        });
        event.sdk = Some(Cow::Owned(sdk));
    }
    Some(event)
}
