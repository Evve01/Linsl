//! The datatypes used throughout the code base.

use std::{collections::HashMap, fmt};

use crate::primitives::{add, car, cdr, eq, gr, inv, mul, neg};

pub type Num = f64;
pub type PosNum = usize;
/// Positions of expressions are tracked and used only to specify where a syntax error has
/// occurred.
pub type Pos = Vec<PosNum>;

/// The basic unit of code in the language. Any valid piece of Linsl code is an expression.
#[derive(Debug, Clone)]
pub enum LinslExpr {
    /// One of '#t' or '#f'.
    Bool(bool),
    /// A lambda function, in the spirit of lambda calculus.
    Closure(Box<LinslExpr>, Box<LinslExpr>),
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
pub enum LinslErr {
    /// Any kind of error not caused by the code, but rather by the interpreter. Should never
    /// occur.
    InternalError(String),
    SyntaxError(String, Pos),
    /// Created if the number of opening parentheses is not the same as closing parentheses.
    /// Returns (number of '(', number of ')')
    UnbalancedParens(PosNum, PosNum),
}

impl fmt::Display for LinslErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            LinslErr::InternalError(s) => s.clone(),
            LinslErr::SyntaxError(s, p) => {
                let mut poss: Vec<String> = p.iter()
                    .map(|c| c.to_string())
                    .collect();
                poss.pop();
                poss.reverse();
                let vec = format!("({})", poss.join(", "));
                format!("Syntax error at {}: {}", vec, s)
            },
            LinslErr::UnbalancedParens(v1, v2) => format!("Unbalanced Parenthesis ({}, {})", v1, v2),
        };

        write!(f, "{}", str)
    }
}

/// The bindings between symbol names and code. The inner scope is the local scope, enabling scoped
/// variables. This enables closures.
#[derive(Debug, Clone)]
pub struct LinslEnv<'a> {
    /// The current local scope.
    pub inner: HashMap<String, LinslExpr>,
    /// The immediate outer scope. Every scope except the global one has an outer scope.
    pub outer: Option<&'a LinslEnv<'a>>,
}

impl LinslEnv<'_> {
    /// The environment when starting the interpreter, i.e. holding only the primitives.
    pub fn default<'a>() -> LinslEnv<'a> {
        let mut env = HashMap::new();

        env.insert("+".to_string(), LinslExpr::Primitive(add));
        env.insert("neg".to_string(), LinslExpr::Primitive(neg));
        env.insert("*".to_string(), LinslExpr::Primitive(mul));
        env.insert("inv".to_string(), LinslExpr::Primitive(inv));
        env.insert("=".to_string(), LinslExpr::Primitive(eq));
        env.insert(">".to_string(), LinslExpr::Primitive(gr));
        env.insert("car".to_string(), LinslExpr::Primitive(car));
        env.insert("cdr".to_string(), LinslExpr::Primitive(cdr));
        LinslEnv { 
            inner: env,
            outer: None,
        }
    }
}
