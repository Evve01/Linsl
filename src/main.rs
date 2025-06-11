//! A simple interpreter for a lisp/scheme like language

use core::fmt;
use std::{collections::HashMap, io::{self, Write}, rc::Rc};

mod primitives;
mod parsing;
mod evaluation;

use evaluation::evaluate;
use parsing::{parse, tokenize};
use primitives::{add, eq, gr, inv, mul, neg};

type Num = f64;

/// The basic unit of code in the language. Any valid piece of Linsl code is an expression.
#[derive(Debug, Clone)]
enum LinslExpr {
    Bool(bool),
    /// A lambda function, in the spirit of lambda calculus.
    Closure(Rc<LinslExpr>, Rc<LinslExpr>),
    List(Vec<LinslExpr>),
    Number(Num),
    /// A built in transformation of expressions. These have deliberately been kept as few as
    /// possible; there are just enough of them to allow other functions that are desirable to be
    /// defined in Linsl.
    Primitive(fn(&[LinslExpr]) -> Result<LinslExpr, LinslErr>),
    Symbol(String),
}

impl fmt::Display for LinslExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            LinslExpr::Bool(b)          => if *b {"#t".to_string()} else {"#f".to_string()}
            LinslExpr::Closure(ps, bd)  => format!("(lambda {}, {})", ps, bd),
            LinslExpr::Primitive(_)     => "Primitive operator".to_string(),
            LinslExpr::List(xs)         => {
                let strs : Vec<String> = xs
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                format!("({})", strs.join(" "))
            }
            LinslExpr::Number(v)        => v.to_string(),
            LinslExpr::Symbol(s)        => s.clone(),
        };

        write!(f, "{}", str)
    }
}

/// Errors that can be encountered when parsing or evaluating code.
#[derive(Debug)]
enum LinslErr {
    InternalError(String),
    ListError(String),
    PrimitivesError(String),
    SyntaxError(u32, u32),
    UnbalancedParens(u32, u32),
}

impl fmt::Display for LinslErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LinslErr::InternalError(s) => s.clone(),
            LinslErr::ListError(s) => s.clone(),
            LinslErr::PrimitivesError(s) => s.clone(),
            LinslErr::SyntaxError(v1, v2) => format!("Syntax error at ({}, {})", v1, v2),
            LinslErr::UnbalancedParens(v1, v2) => format!("Unbalanced Parenthesis at ({}, {})", v1, v2),
        };

        write!(f, "{}", str)
    }
}

/// The current bindings between symbol names and code.
#[derive(Debug, Clone)]
struct LinslEnv<'a> {
    inner: HashMap<String, LinslExpr>,
    outer: Option<&'a LinslEnv<'a>>,
}

impl LinslEnv<'_> {
    /// The environment when starting the interpreter, i.e. holding only the primitives.
    fn default<'a>() -> LinslEnv<'a> {
        let mut env = HashMap::new();

        env.insert("+".to_string(), LinslExpr::Primitive(add));
        env.insert("neg".to_string(), LinslExpr::Primitive(neg));
        env.insert("*".to_string(), LinslExpr::Primitive(mul));
        env.insert("inv".to_string(), LinslExpr::Primitive(inv));
        env.insert("=".to_string(), LinslExpr::Primitive(eq));
        env.insert(">".to_string(), LinslExpr::Primitive(gr));
        LinslEnv { 
            inner: env,
            outer: None,
        }
    }
}

fn parse_eval(expr: String, env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    let (parse_res, _) = parse(&tokenize(expr, 0))?;
    let res = evaluate(&parse_res, env)?;
    Ok(res)
}

fn main() {
    let env = &mut LinslEnv::default();
    loop {
        print!("Linsl> ");
        io::stdout().flush().expect("Could not print!");

        let mut expr = String::new();
        io::stdin().read_line(&mut expr).expect("Failed to read line");
        match parse_eval(expr, env) {
            Ok(res) => println!("{}", res),
            Err(e) => println!("{}", e),
        }
    }
    
}

#[cfg(test)]
mod tests {}
