use std::{any::Any, collections::HashMap};

use anyhow::anyhow;

pub struct ModuleBinder {
    modules: HashMap<&'static str, Box<dyn Any>>,
}

impl ModuleBinder {
    pub(crate) fn new(modules: HashMap<&'static str, Box<dyn Any>>) -> Self {
        Self { modules }
    }
    pub fn get<T: 'static>(&mut self, key: &'static str) -> anyhow::Result<&mut T> {
        match self.modules.get_mut(key) {
            Some(k) => match k.downcast_mut() {
                Some(v) => Ok(v),
                None => Err(anyhow!("failed to convert {} expected type", key)),
            },
            None => Err(anyhow!("failed to find key {}", key)),
        }
    }
}
