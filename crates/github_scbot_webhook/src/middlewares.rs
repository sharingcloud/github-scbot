//! Webhook middlewares.

#![allow(clippy::type_complexity)]

use std::{
    cell::RefCell,
    env,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorUnauthorized, ParseError},
    http::Method,
    web::BytesMut,
    Error, HttpMessage,
};
use futures::{
    future::{ok, Ready},
    stream::StreamExt,
    Future,
};
use tracing::warn;

use super::{
    constants::{
        ENV_DISABLE_SIGNATURE, ENV_GITHUB_SECRET, GITHUB_SIGNATURE_HEADER, SIGNATURE_PREFIX_LENGTH,
    },
    utils::is_valid_signature,
};

/// Signature verification configuration.
pub struct VerifySignature {
    enabled: bool,
    secret: Option<String>,
}

impl VerifySignature {
    /// Create a new configuration.
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for VerifySignature {
    fn default() -> Self {
        let mut enabled = env::var(ENV_DISABLE_SIGNATURE).ok().is_none();
        let secret = if enabled {
            env::var(ENV_GITHUB_SECRET).ok().or_else(|| {
                // Disable signature verification on empty secret
                warn!("Environment variable '{}' is invalid or not set. Disabling signature verification.", ENV_GITHUB_SECRET);
                enabled = false;
                None
            })
        } else {
            warn!("Signature verification is disabled. This can be a security concern.");
            None
        };

        Self { enabled, secret }
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S> for VerifySignature
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = VerifySignatureMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(VerifySignatureMiddleware {
            enabled: self.enabled,
            secret: self.secret.clone(),
            service: Rc::new(RefCell::new(service)),
        })
    }
}

/// Signature verification middleware.
pub struct VerifySignatureMiddleware<S> {
    enabled: bool,
    secret: Option<String>,
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for VerifySignatureMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, mut req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();
        let enabled = self.enabled;
        let secret = self.secret.clone();

        Box::pin(async move {
            if enabled && req.method() == Method::POST {
                if let Some(secret) = secret {
                    let headers = req.headers().clone();
                    let signature = headers
                        .get(GITHUB_SIGNATURE_HEADER)
                        .ok_or_else(|| ErrorUnauthorized(ParseError::Header))?
                        .to_str()
                        .map_err(ErrorUnauthorized)?;

                    // Strip signature prefix
                    let (_, sig) = signature.split_at(SIGNATURE_PREFIX_LENGTH);

                    let mut body = BytesMut::new();
                    let mut stream = req.take_payload();

                    while let Some(chunk) = stream.next().await {
                        body.extend_from_slice(&chunk?);
                    }

                    if !is_valid_signature(sig, &body, &secret) {
                        return Err(ErrorUnauthorized(ParseError::Header));
                    }
                }
            }

            svc.call(req).await
        })
    }
}