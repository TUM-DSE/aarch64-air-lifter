use std::collections::HashMap;

use filecheck::{Value, VariableMap};

#[derive(Debug, Default)]
pub struct SimpleVariableMap {
    variables: HashMap<String, Value<'static>>,
}

impl SimpleVariableMap {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, value: Value<'static>) {
        self.variables.insert(name, value);
    }
}

impl VariableMap for SimpleVariableMap {
    fn lookup(&self, varname: &str) -> Option<Value> {
        self.variables.get(varname).cloned()
    }
}
