use failure::Error;
use std::f64;
use std::f64::consts;

use crate::builtins::BUILTINS;
use crate::env::EnvRef;
use crate::errors::RunError;
// use crate::log;
use crate::values::Value::{self, *};

/// evaluate a structured lisp s-expression
pub fn eval(s_exp: Value, env: EnvRef) -> Result<Value, Error> {
    // log::debug(format!("{:?}", s_exp));
    // log::debug(format!("{}", s_exp.to_string()));

    match s_exp {
        Symbol(ref sym) => {
            if sym.starts_with("'") {
                Ok(Symbol(sym[1..].to_owned()))
            } else {
                Ok(resolve_symbol(sym, env))
            }
        }

        List(list) => {
            if list.len() == 0 {
                Ok(Nil)
            } else {
                run_proc(list, env)
            }
        }

        _ => Ok(s_exp),
    }
}

/// resolve a stored symbol to a value
fn resolve_symbol(symbol: &str, env: EnvRef) -> Value {
    match symbol {
        // touch me not
        "nil"   => Nil,
        "else"  => Bool(true),
        "pi"    => Float(consts::PI),
        "e"     => Float(consts::E),
        "NAN"   => Float(f64::NAN),
        "INF"   => Float(f64::INFINITY),
        "-INF"  => Float(f64::NEG_INFINITY),
        "MAX"   => Float(f64::MAX),
        "MIN"   => Float(f64::MIN),
        _       => env.borrow().get(symbol),
    }
}

/// call a process
fn run_proc(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    let first_element = args.remove(0);

    match first_element {
        Symbol(s) => {
            // check to see if it's a builtin function
            for (name, func) in BUILTINS {
                if &s == name {
                    return func(args, env);
                }
            }

            let first_value = resolve_symbol(&s, env.clone());
            if let Proc(proc) = first_value {
                args = eval_list(args, env.clone())?;
                proc.call(s, args)
            } else {
                Err(RunError::UncallableValue {
                    name: s,
                    typename: first_value.get_type(),
                })?
            }
        }

        List(l) => {
            let result = eval(List(l), env.clone())?;
            args.insert(0, result);
            eval(List(args), env.clone())
        }

        Proc(p) => {
            args = eval_list(args, env.clone())?;
            p.call("<anonymous procedure>".to_owned(), args)
        }

        _ => Err(RunError::UncallableValue {
            name: first_element.to_string(),
            typename: first_element.get_type(),
        })?
    }
}

/// evaluate every Value in a Vec
pub fn eval_list(args: Vec<Value>, env: EnvRef) -> Result<Vec<Value>, Error> {
    args.into_iter().map(|arg| eval(arg, env.clone())).collect()
}
