use std::{any::Any, collections::HashMap};

use anyhow::anyhow;

pub struct ModuleBinder<'a> {
    modules: &'a mut HashMap<&'static str, Box<dyn Any + Send>>,
}

impl<'a> ModuleBinder<'a> {
    pub(crate) fn new(modules: &'a mut HashMap<&'static str, Box<dyn Any + Send>>) -> Self {
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
