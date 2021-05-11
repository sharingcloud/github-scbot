//! Server middlewares.

#![allow(clippy::type_complexity)]

use std::{
    cell::RefCell,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    http::Method,
    web::BytesMut,
    Error, HttpMessage,
};
use futures::{
    future::{ok, Ready},
    stream::StreamExt,
    Future,
};
use github_scbot_conf::Config;
use sentry_actix::eyre::WrapEyre;
use tracing::warn;

use super::{
    constants::{GITHUB_SIGNATURE_HEADER, SIGNATURE_PREFIX_LENGTH},
    utils::is_valid_signature,
};
use crate::ServerError;

/// Signature verification configuration.
pub struct VerifySignature {
    enabled: bool,
    secret: Option<String>,
}

impl VerifySignature {
    /// Create a new configuration.
    pub fn new(config: &Config) -> Self {
        let mut enabled = !config.server_disable_webhook_signature;
        let secret = if enabled {
            if config.github_webhook_secret.is_empty() {
                // Disable signature verification on empty secret
                warn!("Environment variable 'BOT_GITHUB_WEBHOOK_SECRET' is invalid or not set. Disabling signature verification.");
                enabled = false;
                None
            } else {
                Some(config.github_webhook_secret.clone())
            }
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
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Transform = VerifySignatureMiddleware<S>;

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
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;

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
                        .ok_or_else(|| {
                            WrapEyre::unauthorized(ServerError::MissingWebhookSignature)
                        })?
                        .to_str()
                        .map_err(|_e| WrapEyre::forbidden(ServerError::InvalidWebhookSignature))
                        .map_err(|e| {
                            let err: actix_http::Error = e.into();
                            err
                        })?;

                    // Quick check because split_at can panic.
                    if signature.len() <= SIGNATURE_PREFIX_LENGTH {
                        return Err(
                            WrapEyre::forbidden(ServerError::InvalidWebhookSignature).into()
                        );
                    }

                    // Strip signature prefix
                    let (_, sig) = signature.split_at(SIGNATURE_PREFIX_LENGTH);

                    let mut body = BytesMut::new();
                    let mut stream = req.take_payload();

                    while let Some(chunk) = stream.next().await {
                        body.extend_from_slice(&chunk?);
                    }

                    if !is_valid_signature(sig, &body, &secret) {
                        return Err(
                            WrapEyre::forbidden(ServerError::InvalidWebhookSignature).into()
                        );
                    }

                    // Thanks https://github.com/actix/actix-web/issues/1457#issuecomment-617342438
                    let mut payload = actix_http::h1::Payload::empty();
                    payload.unread_data(body.freeze());
                    req.set_payload(payload.into());
                }
            }

            svc.call(req).await
        })
    }
}
