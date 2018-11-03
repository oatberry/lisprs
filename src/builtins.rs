use failure::Error;
use itertools;
use rand;
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;

use crate::env::*;
use crate::eval;
use crate::errors::RunError;
use crate::values::Value::{self, *};
use crate::values::LispProc;

pub const BUILTINS: &[(&str, fn(Vec<Value>, EnvRef) -> Result<Value, Error>)] = &[
    ("define",      define),
    ("undef",       undef),
    ("let",         local_bind),
    ("lambda",      lambda),
    ("if",          if_else),
    ("cond",        cond),
    ("type",        get_type),
    ("quote",       quote),
    ("eval",        eval),
    ("env",         env),
    ("+",           add),
    ("-",           sub),
    ("*",           mul),
    ("/",           div),
    ("modulo",      modulo),
    ("=",           eq),
    ("!=",          neq),
    (">",           gt),
    (">=",          geq),
    ("<",           lt),
    ("<=",          leq),
    ("and",         and),
    ("or",          or),
    ("not",         not),
    ("list-ref",    list_ref),
    ("append",      append),
    ("car",         car),
    ("cdr",         cdr),
    ("length",      length),
    ("cons",        cons),
    ("rand",        rand),
    ("cat",         cat),
    ("uppercase",   uppercase),
    ("lowercase",   lowercase)
];

// {{{ helpful macros
/// return from a function if the Vec $args doesn't contain $num elements
#[macro_export]
macro_rules! check_num_args {
    ($args: ident, $num: expr, $name: expr) => {{
        if $args.len() != $num {
            Err(RunError::WrongNumArgs {
                name: $name.to_string(),
                expected: $num,
                got: $args.len(),
            })
        } else {
            Ok(())
        }
    }}
}

/// extract the inner Rust type value from a lisp value, returning an Err
/// if $value is not of enum variant $variant
macro_rules! extract {
    ($value: expr, $variant:path, $proc: expr) => {{
        if let $variant(x) = $value {
            Ok(x)
        } else {
            Err(RunError::TypeError {
                name: $proc.to_string(),
                expected: stringify!($variant).to_string(),
                got: $value.get_type(),
            })
        }
    }};

    ($value: expr, &$variant:path, $proc: expr) => {{
        if let &$variant(ref a) = $value {
            Ok(a.clone())
        } else {
            Err(RunError::TypeError {
                name: $proc.to_string(),
                expected: stringify!($variant).to_string(),
                got: $value.get_type(),
            })
        }
    }}
}

/// return a Err(RunError::ProcError)
macro_rules! procerr {
    ($name: expr, $msg: expr) => {
        Err(RunError::ProcError {
            name: $name.to_string(),
            msg: $msg.to_string(),
        }.into())
    }
}

/// return a success message
macro_rules! success {
    () => {
        Ok(Value::Str("success".to_string()))
    }
}
// }}}

// {{{ essentials
/// save a value to the Env
/// usage: (define <symbol> <value>)
///        (define (<func-name> param1 param2) (<body-expr>))
pub fn define(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "define")?;

    match &args[0] {
        Symbol(var_name) => {
            let expr = args[1].clone();
            let expr_result = eval::eval(expr, env.clone())?;
            env.borrow_mut().define(var_name, expr_result);
            success!()
        },

        List(list) => {
            let mut list = list.clone();
            if list.len() < 1 {
                return procerr!("define", "cannot define an empty list");
            }

            let proc_name: String = extract!(list.remove(0), Symbol, "define")?;
            let body = args[1].clone();
            let proc = lambda(vec![List(list.to_vec()), body], env.clone())?;
            env.borrow_mut().define(&proc_name, proc);
            success!()
        },

        _ => procerr!("define", "only symbols can be redefined")
    }
}

/// remove a definition from the Env
/// usage: (undef <symbol>)
pub fn undef(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "undef")?;

    let var_name: String = extract!(&args[0], &Symbol, "undef")?;
    env.borrow_mut().undefine(&var_name);
    success!()
}

/// evaluate an expression with local, unsaved bindings
/// usage: (let ((<symbol> <expr>)
///              (<symbol> <expr>)
///              (...))
///             <expr>)
pub fn local_bind(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "let")?;

    let mut local_env = Env::new(Some(env.clone()));
    let bindings: Vec<Value> = extract!(&args[0], &List, "let")?;

    for bind in bindings {
        let bind: Vec<Value> = extract!(&bind, &List, "let (in binds list)")?;
        check_num_args!(bind, 2, "let (in binding)")?;

        let var_name: String = extract!(&bind[0], &Symbol, "let (in binding)")?;
        let expr = &bind[1];
        let expr_result = eval::eval(expr.clone(), env.clone())?;

        local_env.define(&var_name, expr_result);
    }

    let local_env_ref: EnvRef = Rc::new(RefCell::new(local_env));
    eval::eval(args[1].clone(), local_env_ref)
}

/// create a function
/// usage: (lambda (param1 param2...) <body-expr>)
pub fn lambda(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "lambda")?;

    let params: Vec<Value> = extract!(&args[0], &List, "lambda")?;
    let mut param_names: Vec<String> = Vec::with_capacity(params.len());

    for p in params {
        let param_name = extract!(p, Symbol, "lambda (in params)")?;
        param_names.push(param_name);
    }

    let body = args[1].clone();
    Ok(Proc(box LispProc {
        params: param_names,
        body,
        env: env.clone(),
    }))
}

/// conditionally evaluate an expression
/// usage: (if <bool-expr> <conseq-expr> <alternate-expr>)
pub fn if_else(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 3, "if")?;

    let test = eval::eval(args[0].clone(), env.clone())?.to_bool();
    let conseq = args[1].clone();
    let alt = args[2].clone();

    eval::eval(if test { conseq } else { alt }, env.clone())
}

/// conditionally evaluate an expression (like branching)
/// usage: (cond (<bool-expr> <conseq-expr>)
///              (<bool-expr> <conseq-expr>)
///              (...)
///              (else <alternate-expr>))
pub fn cond(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    if args.len() < 1 {
        return procerr!("cond", "at least 1 branch required");
    }

    for branch in args {
        let branch: Vec<Value> = extract!(branch, List, "cond")?;
        check_num_args!(branch, 2, "cond (in branch)")?;

        if eval::eval(branch[0].clone(), env.clone())?.to_bool() {
            return eval::eval(branch[1].clone(), env.clone());
        }
    }

    procerr!("cond", "no branches evaluated and no `else` branch found")
}

/// return the the type of a value as a str
/// usage: (type <expr>)
pub fn get_type(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "type")?;

    let thingtype = eval::eval(args.pop().unwrap(), env.clone())?.get_type();
    Ok(Str(thingtype))
}

/// return an expression without evaluating it
/// usage: (quote <expr>)
///        '<expr>
pub fn quote(mut args: Vec<Value>, _env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "quote")?;
    Ok(args.pop().unwrap())
}

/// evaluate an sexp
/// usage: (eval <expr>)
pub fn eval(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "eval")?;
    args = eval::eval_list(args, env.clone())?;
    eval::eval(args[0].clone(), env)
}

/// return a list of all defined symbols
/// usage: (env)
pub fn env(_args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    Ok(List(env.borrow().vars.keys().map(|s| Symbol(s.to_owned())).collect()))
}
// }}}

// {{{ math
/// do some math
/// usage: (+ <num> <num>)
///        (- <num> <num>)
///        (* <num> <num>)
///        (/ <num> <num>)
///        (^ <num> <num>)
fn math(op: &str, mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    if args.len() < 2 {
        return procerr!(op, "at least 2 arguments required")
    }

    args = eval::eval_list(args, env)?;
    // make sure all arguments are floats or integers
    for arg in &args {
        match arg {
            Integer(_) => continue,
            Float(_) => continue,
            _ => return Err(RunError::TypeError {
                name: op.to_string(),
                expected: "number".to_string(),
                got: arg.get_type(),
            }.into())
        }
    }

    let init = args.remove(0);
    let result = match op {
        "+" => args.into_iter().fold(init, |acc, n| acc + n),
        "-" => args.into_iter().fold(init, |acc, n| acc - n),
        "*" => args.into_iter().fold(init, |acc, n| acc * n),
        "/" => args.into_iter().fold(init, |acc, n| acc / n),
        "%" => init % args.remove(0),
        _ => panic!("'{}' is not a valid math operator, check your code! "),
    };

    Ok(result)
}

pub fn add(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    math("+", args, env)
}

pub fn sub(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    math("-", args, env)
}

pub fn mul(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    math("*", args, env)
}

pub fn div(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    math("/", args, env)
}

pub fn modulo(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    math("%", args, env)
}
// }}}

// {{{ logic
/// do some boolean logic
/// usage: (= <expr> <expr>)
///        (!= <expr> <expr>)
///        (> <num> <num>)
///        (>= <num> <num>)
///        (< <num> <num>)
///        (<= <num> <num>)
///        (and <bool> <bool>)
///        (or <bool> <bool>)
fn logic(op: &str, mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, format!("logic op `{}`", op))?;

    args = eval::eval_list(args, env)?;
    match op {
        "="   => Ok(Bool(args[0] == args[1])),
        "!="  => Ok(Bool(args[0] != args[1])),
        ">"   => Ok(Bool(args[0] >  args[1])),
        ">="  => Ok(Bool(args[0] >= args[1])),
        "<"   => Ok(Bool(args[0] <  args[1])),
        "<="  => Ok(Bool(args[0] <= args[1])),
        "and" => Ok(Bool(args[0].to_bool() && args[1].to_bool())),
        "or"  => Ok(Bool(args[0].to_bool() || args[1].to_bool())),
        _     => panic!("{} is not a valid comparison operator", op),
    }
}

pub fn eq(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("=", args, env)
}

pub fn neq(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("!=", args, env)
}

pub fn gt(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic(">", args, env)
}

pub fn geq(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic(">=", args, env)
}

pub fn lt(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("<", args, env)
}

pub fn leq(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("<=", args, env)
}

pub fn and(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("and", args, env)
}

pub fn or(args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    logic("or", args, env)
}

/// return the logical inverse of a bool
/// usage: (not <bool>)
pub fn not(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "not")?;

    args = eval::eval_list(args, env)?;
    Ok(Bool(!args[0].to_bool()))
}
// }}}

// {{{ lists
/// construct a list
/// usage: (cons <value> <list>)
pub fn cons(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "cons")?;

    args = eval::eval_list(args, env)?;
    let mut list: Vec<Value> = extract!(args.pop().unwrap(), List, "cons")?;
    let value = args.pop().unwrap();

    list.insert(0, value);
    Ok(List(list))
}

/// get the length of a list or a string
/// usage: (length <list>)
///        (length <str>)
pub fn length(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "length")?;

    args = eval::eval_list(args, env)?;
    match args[0] {
        List(ref l) => Ok(Integer(l.len() as i64)),
        Str(ref s)  => Ok(Integer(s.chars().count() as i64)),
        _ => procerr!("length", format!("expected a `List` or `Str`, got a {} instead",
                                        args[0].get_type())),
    }
}

/// get the item at an index in a list
/// usage: (list_ref <list> <ref>)
pub fn list_ref(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "list-ref")?;

    args = eval::eval_list(args, env)?;
    let list: Vec<Value> = extract!(&args[0], &List, "list-ref")?;
    let idx = extract!(args[1], Integer, "list-ref")?;
    list.get(idx as usize - 1)
        .cloned()
        .ok_or(RunError::IndexOutOfBounds(idx as usize).into())
}

/// concatenate two lists together
/// usage: (append <list> <list>)
pub fn append(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 2, "append")?;

    args = eval::eval_list(args, env)?;
    let mut list1 = extract!(&args[0], &List, "append")?;
    let mut list2 = extract!(&args[1], &List, "append")?;
    list1.append(&mut list2);
    Ok(List(list1))
}

/// return the first element of a populated list, or nil
/// usage: (car <list>)
pub fn car(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "car")?;

    args = eval::eval_list(args, env)?;
    let list = extract!(&args[0], &List, "car")?;
    Ok(list.get(0).cloned().unwrap_or(Nil))
}

/// return all elements of a list but the first
/// usage: (cdr <list>)
pub fn cdr(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "cdr")?;

    args = eval::eval_list(args, env)?;
    let list = extract!(&args[0], &List, "cdr")?;
    Ok(List(list.get(1..).unwrap_or(&[]).to_vec()))
}

/// return a random argument or a random element of a list
/// usage: (rand <list>)
///        (rand <expr> <expr> ...)
pub fn rand(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    if args.len() == 0 {
        procerr!("rand", "at least 1 argument required")
    } else if args.len() > 1 {
        args = eval::eval_list(args, env)?;
        Ok(rand::thread_rng().choose(&args).cloned().unwrap_or(Nil))
    } else {
        args = eval::eval_list(args, env)?;
        let list = extract!(&args[0], &List, "rand")?;
        Ok(rand::thread_rng().choose(&list).cloned().unwrap_or(Nil))
    }
}
// }}}

// {{{ strings
/// concatenate values together into a string
/// usage: (cat <value> <value> ...)
pub fn cat(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    args = eval::eval_list(args, env.clone())?;
    Ok(Str(itertools::join(args, "")))
}

/// translate the characters in a string to uppercase
/// usage: (uppercase <str>)
pub fn uppercase(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "uppercase")?;
    args = eval::eval_list(args, env)?;

    let string = extract!(&args[0], &Str, "uppercase")?;
    Ok(Str(string.to_uppercase()))
}

/// translate the characters in a string to lowercase
/// usage: (lowercase <str>)
pub fn lowercase(mut args: Vec<Value>, env: EnvRef) -> Result<Value, Error> {
    check_num_args!(args, 1, "lowercase")?;
    args = eval::eval_list(args, env)?;

    let string = extract!(&args[0], &Str, "lowercase")?;
    Ok(Str(string.to_lowercase()))
}
// }}}
