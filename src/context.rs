use std::sync::Arc;

#[derive(Clone)]
pub struct Context {}

impl Context {
    /// Creates a new top-level context; this function should only
    /// be at the origin of the request.
    pub fn new() -> Self {
        Self {}
    }

    pub fn with_cancel(&self) -> (Context, Box<dyn Fn()>) {
        unimplemented!()
    }

    pub fn with_deadline(&self) -> (Context, Box<dyn Fn()>) {
        unimplemented!()
    }

    // non-blocking check to see if this context is done and the thread should be cancelled
    pub fn is_done(&self) -> bool {
        unimplemented!()
    }
}
