use std::{thread, time::Duration, sync::Arc};

use log::error;

use crate::Context;

pub struct ServerHealth {
    on_healthy_callbacks: Vec<Box<dyn FnOnce(Arc<Context>) -> anyhow::Result<()> + Send>>,
    on_shutdown_callbacks: Vec<Box<dyn FnOnce(Arc<Context>) -> anyhow::Result<()> + Send>>,
}

impl ServerHealth {
    pub(crate) fn new() -> Self {
        Self {
            on_healthy_callbacks: vec![],
            on_shutdown_callbacks: vec![],
        }
    }

    pub fn run(self, ctx: Arc<Context>) -> anyhow::Result<()> {
        let mut threads = vec![];
        // Spin off all healthy callbacks
        for cb in self.on_healthy_callbacks {
            let ctx = ctx.clone();
            threads.push(thread::spawn(move || (cb)(ctx)));
        }
        // TODO: handle SIGINT or SIGSTOP
        // Wait till context gets cancelled
        loop {
            if ctx.is_done() {
                break;
            }
            // check if any cb's resolved
            let mut i = 0;
            while i < threads.len() {
                if threads[i].is_finished() {
                    if let Err(e) = threads.remove(i).join() {
                        error!("{:?}", e);
                        ctx.cancel();
                    }
                } else {
                    i += 1;
                }
            }
            thread::sleep(Duration::from_millis(250));
        }
        // Wait for all callbacks to exit
        for th in threads {
            if let Err(e) = th.join() {
                error!("{:?}", e);
            }
        }
        let ctx = Context::new();
        let mut threads = vec![];
        for cb in self.on_shutdown_callbacks {
            let ctx = ctx.clone();
            threads.push(thread::spawn(move || (cb)(ctx)));
        }
        Ok(())
    }

    pub fn register_on_healthy<F>(&mut self, cb: F)
    where
        F: FnOnce(Arc<Context>) -> anyhow::Result<()> + Send + 'static,
    {
        self.on_healthy_callbacks.push(Box::new(cb))
    }

    pub fn register_on_shutdown<F>(&mut self, cb: F)
    where
        F: FnOnce(Arc<Context>) -> anyhow::Result<()> + Send + 'static,
    {
        self.on_shutdown_callbacks.push(Box::new(cb))
    }
}
