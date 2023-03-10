use std::any::Any;

use crate::{module_binder::ModuleBinder, server_health::ServerHealth};

pub trait Configurator: Send {
    fn depends_on(&self) -> Vec<&'static str>;
    fn configure(
        &mut self,
        module: Box<dyn Any>,
        binder: &mut ModuleBinder,
        server: &mut ServerHealth,
    ) -> anyhow::Result<()>;
}
