use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::values::Value;

/// The “memory” of the interpreter is represented as a HashMap, with an
/// optional parent EnvRef, that is passed around in an Rc<Refcell<>>,
/// which allows for multiple “owners” with interior mutability.
#[derive(Debug, Clone)]
pub struct Env {
    pub vars: HashMap<String, Value>,
    pub parent: Option<EnvRef>,
}

/// an interior-mutable, reference-counted smart pointer wrapper around an `Env`
pub type EnvRef = Rc<RefCell<Env>>;

impl Env {
    /// create a new lisprs environment
    pub fn new(parent: Option<EnvRef>) -> Env {
        Env {
            vars: HashMap::new(),
            parent,
        }
    }

    /// resolve a symbol to a stored lisprs value, returning
    /// itself as a string if no stored value is found
    pub fn get(&self, var_name: &str) -> Value {
        match self.vars.get(var_name) {
            Some(x) => x.clone(),
            None => {
                // try to find the var in the parent
                match &self.parent {
                    Some(env) => env.borrow().get(var_name),
                    None      => Value::Str(var_name.to_owned()),
                }
            }
        }
    }

    /// add (or modify) a stored value in the environment
    pub fn define(&mut self, var_name: &str, value: Value) {
        self.vars.insert(var_name.to_owned(), value);
    }

    /// remove a stored value from the environment
    pub fn undefine(&mut self, var_name: &str) {
        self.vars.remove(var_name);
    }
}
