use crate::{module_binder::ModuleBinder, server_health::ServerHealth};

pub trait Configurator {
    fn depends_on(&self) -> Vec<&'static str>;
    fn configure(
        &mut self,
        binder: &mut ModuleBinder,
        server: &mut ServerHealth,
    ) -> anyhow::Result<()>;
}
