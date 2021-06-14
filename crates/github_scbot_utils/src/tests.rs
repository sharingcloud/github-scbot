//! Test utils.

use std::sync::RwLock;

#[derive(Default)]
pub(crate) struct MockInternal {
    call_count: u64,
}

/// Simple mock.
pub struct Mock<Response: Clone> {
    internal: RwLock<MockInternal>,
    response: Response,
}

impl<Response: Clone> Mock<Response> {
    /// Creates a new mock.
    pub fn new(response: Response) -> Self {
        Self {
            internal: RwLock::new(MockInternal::default()),
            response,
        }
    }

    fn increment_call_count(&self) {
        self.internal.write().unwrap().call_count += 1;
    }

    /// Checks if mock has been called.
    pub fn called(&self) -> bool {
        self.internal.read().unwrap().call_count > 0
    }

    /// Checks mock call count.
    pub fn call_count(&self) -> u64 {
        self.internal.read().unwrap().call_count
    }

    /// Gets mock response.
    pub fn response(&self) -> Response {
        self.increment_call_count();
        self.response.clone()
    }

    /// Sets mock response.
    pub fn set_response(&mut self, r: Response) {
        self.response = r;
    }
}
