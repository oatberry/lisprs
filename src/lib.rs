#![feature(box_patterns, box_syntax)]

#[macro_use]
extern crate failure_derive;

mod arithmetic;
mod builtins;
mod env;
mod eval;
mod errors;
mod file;
mod log;
mod parser;
pub mod values;

use failure::Error;
use std::cell::RefCell;
use std::rc::Rc;

use crate::values::Value;
use crate::env::*;

#[derive(Clone)]
pub struct Interpreter {
    pub env: EnvRef,
}

impl Interpreter {
    /// evaluate a string as lisp code
    pub fn run<S: Into<String>>(&self, code: S) -> Result<Value, Error> {
        // parse into an s-expression
        let sexp = Value::new(code.into())?;

        // log::debug(format!("{:?}", sexp));
        eval::eval(sexp, self.env.clone())
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Interpreter { env: Rc::new(RefCell::new(Env::new(None))) }
    }
}

// {{{ tests
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
// }}}
