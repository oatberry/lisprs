use failure::Error;
use itertools::join;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

use crate::env::*;
use crate::eval;
use crate::errors::*;
use crate::parser::{self, Token};

/// representation of lisprs' data types
#[derive(Debug, Clone)]
pub enum Value {
    Symbol(String),
    Str(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    List(Vec<Value>),
    Proc(Box<LispProc>),
    Nil,
}

use self::Value::*;

impl Value {
    /// parse a string into a structured s-expression
    pub fn new(s: String) -> Result<Self, Error> {
        let mut tokens = parser::tokenize(&s);
        let left_parens = tokens.iter().filter(|&t| t == &Token::LeftParen).count();
        let right_parens = tokens.iter().filter(|&t| t == &Token::RightParen).count();

        if tokens.is_empty() {
            Err(ParseError::Empty)?
        } else if left_parens == right_parens {
            Value::from_tokens(&mut tokens)
        } else {
            Err(ParseError::MismatchedParens)?
        }
    }

    /// represent a `Value` as a human-friendly string
    pub fn to_string(&self) -> String {
        match self {
            Symbol(s)   => s.clone(),
            Str(s)      => s.to_owned(),
            Integer(n)  => n.to_string(),
            Float(n)    => n.to_string(),
            Bool(true)  => "#t".to_owned(),
            Bool(false) => "#f".to_owned(),
            Nil         => "nil".to_owned(),
            List(list)  => format!(
                "({})",
                join(list.iter().map(|item| item.serialize()), " ")
            ),
            Proc(proc)  => format!(
                "(lambda ({}) {})",
                join(proc.params.iter(), " "),
                proc.body.to_string()
            ),
        }
    }

    /// represent a `Value` as a slightly less human-friendly string for saving externally
    pub fn serialize(&self) -> String {
        match self {
            Symbol(s)   => s.clone(),
            Str(s)      => format!("\"{}\"", s),
            Integer(n)  => n.to_string(),
            Float(n)    => n.to_string(),
            Bool(true)  => "#t".to_owned(),
            Bool(false) => "#f".to_owned(),
            Nil         => "nil".to_owned(),
            List(list)  => format!(
                "({})",
                join(list.iter().map(|item| item.serialize()), " ")
            ),
            Proc(proc)  => format!(
                "(lambda ({}) {})",
                join(proc.params.iter(), " "),
                proc.body.serialize()
            ),
        }
    }

    /// make a bool out of a value. nil, empty list, and 0 are falsy.
    pub fn to_bool(&self) -> bool {
        match self {
            Bool(b)    => *b,
            Nil        => false,
            List(l)    => l.is_empty(),
            Integer(n) => *n != 0i64,
            Float(n)   => *n != 0f64,
            _          => true,
        }
    }

    /// get the human-friendly type of a `Value`
    pub fn get_type(&self) -> String {
        match self {
            Symbol(_)  => "Symbol",
            Str(_)     => "Str",
            Integer(_) => "Integer",
            Float(_)   => "Float",
            Bool(_)    => "Bool",
            List(_)    => "List",
            Proc(_)    => "Proc",
            Nil        => "Nil",
        }.to_owned()
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Bool(a), Bool(b))       => a == b,
            (Integer(a), Integer(b)) => a == b,
            (Float(a), Float(b))     => a == b,
            (Integer(a), Float(b))   => &(*a as f64) == b,
            (Float(a), Integer(b))   => a == &(*b as f64),
            (Symbol(a), Symbol(b))   => a == b,
            (Str(a), Str(b))         => a == b,
            (Nil, Nil)               => true,
            _ => false, // values of different types are not equivalent
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Integer(a), Integer(b)) => a.partial_cmp(b),
            (Float(a), Float(b))     => a.partial_cmp(b),
            (Integer(a), Float(b))   => (*a as f64).partial_cmp(b),
            (Float(a), Integer(b))   => a.partial_cmp(&(*b as f64)),
            _ => None
        }
    }
}

/// a lisp process (or “function”), represented as a list of named, typeless paramaters,
/// a yet un-evaluated s-expression, and an EnvRef
#[derive(Debug, Clone)]
pub struct LispProc {
    pub params: Vec<String>,
    pub body: Value,
    pub env: EnvRef,
}

impl LispProc {
    /// run a LispProc with some arguments
    pub fn call(&self, name: String, mut args: Vec<Value>) -> Result<Value, Error> {
        // let mut args = eval::eval_list(args, self.env.clone())?;
        // log::debug(format!("calling {} with args: {:?}", name, args));

        if !self.params.contains(&".".to_owned()) && (args.len() != self.params.len()) {
            Err(RunError::WrongNumArgs {
                name,
                expected: self.params.len(),
                got: args.len(),
            })?
        }

        let mut local_env = Env::new(Some(self.env.clone()));

        let mut i = 0;
        while i < self.params.len() {
            if self.params[i] == "." {
                i += 1;
                local_env.define(&self.params[i], Value::List(args));
                break;
            }

            local_env.define(&self.params[i], args.remove(0));
            i += 1;
        }

        let local_env_ref: EnvRef = Rc::new(RefCell::new(local_env));
        eval::eval(self.body.clone(), local_env_ref)
    }
}
