mod client;
mod debug;

pub use client::with_sentry_configuration;
pub use debug::send_test_event;
pub use sentry;
pub use sentry_actix as actix;
pub use sentry_actix::WrapEyre;
pub use sentry_eyre::{capture_eyre, eyre, EyreHubExt};
