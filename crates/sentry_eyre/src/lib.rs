//! Adds support for capturing Sentry errors from [`eyre::Result`].
//!
//! This integration adds a new event *source*, which allows you to create events directly
//! from an [`eyre::Result`] struct.  As it is only an event source it does not need to be enabled in the call to
//! [`sentry::init`](https://docs.rs/sentry/*/sentry/fn.init.html).
//!
//! This integration does not need to be installed, instead it provides an extra function to
//! capture [`eyre::Result`], optionally exposing it as a method on the
//! [`sentry::Hub`](https://docs.rs/sentry/*/sentry/struct.Hub.html) using the
//! [`EyreHubExt`] trait.
//!
//! Like a plain [`std::error::Error`] being captured, [`eyre::Result`] is captured with a
//! chain of all error sources, if present.  See
//! [`sentry::capture_error`](https://docs.rs/sentry/*/sentry/fn.capture_error.html) for
//! details of this.
//!
//! # Example
//!
//! ```ignore
//! use sentry_eyre::capture_eyre;
//!
//! fn function_that_might_fail() -> eyre::Result<()> {
//!     Err(eyre::eyre!("some kind of error"))
//! }
//!
//! if let Err(err) = function_that_might_fail() {
//!     capture_eyre(&err);
//! }
//! ```
//!
//! [`eyre::Error`]: https://docs.rs/eyre/*/eyre/struct.Report.html

#![doc(html_favicon_url = "https://sentry-brand.storage.googleapis.com/favicon.ico")]
#![doc(html_logo_url = "https://sentry-brand.storage.googleapis.com/sentry-glyph-black.png")]
#![warn(missing_docs)]
#![deny(unsafe_code)]

use sentry_backtrace::{backtrace_to_stacktrace, process_event_stacktrace};
use sentry_core::{types::Uuid, Hub};
use stable_eyre::{eyre, BacktraceExt};

/// Captures an [`eyre::Report`].
///
/// This will capture an eyre report as a sentry event if a
/// [`sentry::Client`](../../struct.Client.html) is initialised, otherwise it will be a
/// no-op.  The event is dispatched to the thread-local hub, with semantics as described in
/// [`Hub::current`].
///
/// See [module level documentation](index.html) for more information.
///
/// [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
pub fn capture_eyre(e: &eyre::Report) -> Uuid {
    Hub::with_active(|hub| hub.capture_eyre(e))
}

/// Hub extension methods for working with [`eyre`].
///
/// [`eyre`]: https://docs.rs/eyre
pub trait EyreHubExt {
    /// Captures an [`eyre::Report`] on a specific hub.
    ///
    /// [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
    fn capture_eyre(&self, e: &eyre::Report) -> Uuid;
}

impl EyreHubExt for Hub {
    fn capture_eyre(&self, e: &eyre::Report) -> Uuid {
        let err: &dyn std::error::Error = e.as_ref();
        let mut evt = sentry_core::event_from_error(err);

        // Add traceback
        if let Some(bt) = e.backtrace() {
            if let Some(mut st) = backtrace_to_stacktrace(bt) {
                if let Some(client) = self.client() {
                    process_event_stacktrace(&mut st, client.options());
                }

                if let Some(mut exc) = evt.exception.last_mut() {
                    exc.stacktrace = Some(st);
                }
            }
        }

        self.capture_event(evt)
    }
}
