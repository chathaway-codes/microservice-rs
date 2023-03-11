use std::{
    any::Any,
    collections::{HashMap, HashSet}, sync::Arc,
};

use anyhow::bail;

use crate::{configurator::Configurator, ModuleBinder, ServerHealth, Context};

pub struct ModuleCollector {
    modules: HashMap<&'static str, Box<dyn Any + Send>>,
    configurators: HashMap<&'static str, Box<dyn Configurator>>,
}

impl ModuleCollector {
    pub fn new() -> Self {
        Self {
            modules: HashMap::default(),
            configurators: HashMap::default(),
        }
    }

    /// register a module with this collector; most modules provide a `Type::register` function which wraps
    /// this call.
    ///
    /// NOTE: is is important the module key be unique. Ideally, it will be a string related to the crate/file:line
    ///
    /// Example:
    /// ```
    /// use std::any::Any;
    /// use microservice_rs::{ModuleCollector, Configurator, ModuleBinder, ServerHealth};
    ///
    /// const MODULE_NAME: &str = "mycrate/modules/http_server.rs:23";
    /// pub struct MyModule {}
    /// struct MyModuleConfigurator {}
    ///
    /// impl MyModule {
    ///     pub fn register(collector: &mut ModuleCollector) {
    ///         collector.register(MODULE_NAME, Self{}, MyModuleConfigurator{})
    ///     }
    /// }
    ///
    /// impl Configurator for MyModuleConfigurator {
    ///     fn configure(&mut self, _module: Box<dyn Any>, _binder: &mut ModuleBinder, _server: &mut ServerHealth) -> anyhow::Result<()> {
    ///         Ok(())
    ///     }
    ///     fn depends_on(&self) -> Vec<&'static str> {
    ///         vec!()
    ///     }
    /// }
    /// ```
    ///
    pub fn register<T: Any + Send, C: Configurator + 'static>(
        &mut self,
        key: &'static str,
        value: T,
        configurator: C,
    ) {
        if self.modules.contains_key(&key) {
            return;
        }
        self.modules.insert(key, Box::new(value));
        self.configurators.insert(key, Box::new(configurator));
    }

    /// start configures the modules and runs till all health functions exit.
    pub fn start(self, ctx: Arc<Context>) -> anyhow::Result<()> {
        let order = self.config_order()?;
        let (mut binder, mut configurators) = (self.modules, self.configurators);
        let mut server = ServerHealth::new();
        for k in order {
            let module = binder.remove(k).unwrap();
            let mut binder = ModuleBinder::new(&mut binder);
            configurators
                .remove(k)
                .unwrap()
                .configure(module, &mut binder, &mut server)?;
        }
        server.run(ctx)?;
        Ok(())
    }

    // config_order returns the order to configure things such that configure is called
    // after everything that depends on a module has had a chance to register with it.
    fn config_order(&self) -> anyhow::Result<Vec<&'static str>> {
        // this module is blocking these other modules; it can only be added to the order
        // when the blocking set is empty
        let mut blocked_by: HashMap<&'static str, HashSet<&'static str>> =
            HashMap::with_capacity(self.modules.len());
        let mut inverse_blocked_by: HashMap<&'static str, HashSet<&'static str>> =
            HashMap::with_capacity(self.modules.len());
        let mut order = Vec::with_capacity(self.modules.len());

        // collect all the dependencies
        for (key, module) in self.configurators.iter() {
            for dep in module.depends_on() {
                blocked_by
                    .entry(dep)
                    .or_insert_with(HashSet::new)
                    .insert(key);
                inverse_blocked_by
                    .entry(key)
                    .or_insert_with(HashSet::new)
                    .insert(dep);
            }
            blocked_by.entry(key).or_insert_with(HashSet::new);
            inverse_blocked_by
                .entry(key)
                .or_insert_with(HashSet::new);
        }

        // insert them
        while !blocked_by.is_empty() {
            let mut to_delete = vec![];
            // TODO: we can improve the performance here by keeping track which of things are unblocked
            // rather than iterating over all of them
            for (key, set) in blocked_by.iter() {
                if set.is_empty() {
                    order.push(*key);
                    to_delete.push(*key);
                }
            }
            if to_delete.is_empty() {
                bail!(format!(
                    "stuck resolving dependencies; remaining: {blocked_by:?}"
                ))
            }
            for k in to_delete {
                // unwrap here is safe because we know that k exists in the inverse_blocked_by_set due to above code.
                for inverse in inverse_blocked_by.get(k).unwrap().iter() {
                    blocked_by.get_mut(*inverse).unwrap().remove(k);
                }
                blocked_by.remove(k);
            }
        }
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MOD1: &str = "mod1";
    const MOD2: &str = "mod2";

    #[test]
    fn register_modules() {
        let mut collector = ModuleCollector::new();

        struct Mod1 {}
        struct Mod1Config {}
        impl Configurator for Mod1Config {
            fn configure(
                &mut self,
                _module: Box<dyn Any>,
                _binder: &mut crate::module_binder::ModuleBinder,
                _server: &mut crate::server_health::ServerHealth,
            ) -> anyhow::Result<()> {
                Ok(())
            }
            fn depends_on(&self) -> Vec<&'static str> {
                vec![]
            }
        }
        struct Mod2 {}
        struct Mod2Config {}
        impl Configurator for Mod2Config {
            fn configure(
                &mut self,
                _module: Box<dyn Any>,
                _binder: &mut crate::module_binder::ModuleBinder,
                _server: &mut crate::server_health::ServerHealth,
            ) -> anyhow::Result<()> {
                Ok(())
            }

            fn depends_on(&self) -> Vec<&'static str> {
                vec![MOD1]
            }
        }

        let (mod1, mod2) = (Mod1 {}, Mod2 {});

        collector.register(MOD1, mod1, Mod1Config {});
        collector.register(MOD2, mod2, Mod2Config {});

        assert_eq!(collector.modules.len(), 2);

        let ctx = Context::new();
        ctx.cancel();

        collector.start(ctx).unwrap();
    }
}
