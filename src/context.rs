use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Context {
    inner: Arc<RwLock<Inner>>,
}

impl Context {
    /// Creates a new top-level context; this function should only
    /// be at the origin of the request.
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(RwLock::new(Inner{
                children: vec!(),
                done: false,
            })),
        })
    }

    pub fn with_cancel(&self) -> Arc<Context> {
        let next = Self::new();
        let mut inner = self.inner.write().unwrap();
        inner.children.push(next.clone());
        next
    }

    pub fn with_deadline(&self) -> Arc<Context> {
        unimplemented!()
    }

    // non-blocking check to see if this context is done and the thread should be cancelled
    pub fn is_done(&self) -> bool {
        self.inner.read().unwrap().done
    }

    pub fn cancel(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.done = true;
    }
}

struct Inner {
    children: Vec<Arc<Context>>,
    done: bool,
}
