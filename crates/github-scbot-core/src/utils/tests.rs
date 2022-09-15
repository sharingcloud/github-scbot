//! Test utils.

use std::sync::RwLock;

#[derive(Default)]
pub(crate) struct MockInternal {
    call_count: u64,
}

/// Simple mock.
pub struct Mock<Args, Output> {
    internal: RwLock<MockInternal>,
    cb: Box<dyn Fn(Args) -> Output + Send + Sync>,
}

impl<Args, Output> Mock<Args, Output> {
    /// Creates a new mock.
    pub fn new(cb: Box<dyn Fn(Args) -> Output + Send + Sync>) -> Self {
        Self {
            internal: RwLock::new(MockInternal::default()),
            cb,
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
    pub fn call(&self, args: Args) -> Output {
        self.increment_call_count();
        (self.cb)(args)
    }

    /// Sets mock response.
    pub fn set_callback(&mut self, f: Box<dyn Fn(Args) -> Output + Send + Sync>) {
        self.cb = f;
    }
}

#[allow(clippy::module_inception)]
#[cfg(test)]
mod tests {
    use super::Mock;

    #[test]
    fn test_mock() {
        let mut mock: Mock<u16, u16> = Mock::new(Box::new(|x| x * 2));
        assert_eq!(mock.call(2), 4);
        assert!(mock.called());
        assert_eq!(mock.call_count(), 1);

        mock.set_callback(Box::new(|x| x * 4));
        assert_eq!(mock.call(2), 8);
        assert_eq!(mock.call_count(), 2);
    }

    #[test]
    fn test_set_callback() {
        #[derive(Clone, Debug, PartialEq)]
        struct MyStruct {
            pub a: String,
            pub b: String,
        }

        let mut mock: Mock<MyStruct, MyStruct> = Mock::new(Box::new(|mut x| {
            x.a = "Pouet".to_string();
            x
        }));

        let struct_test_input = MyStruct {
            a: "1".to_string(),
            b: "2".to_string(),
        };
        let struct_test = MyStruct {
            a: "A".to_string(),
            b: "B".to_string(),
        };
        let struct_test_clone = struct_test.clone();

        mock.set_callback(Box::new(move |_| struct_test_clone.clone()));
        assert_eq!(mock.call(struct_test_input), struct_test);
    }
}
