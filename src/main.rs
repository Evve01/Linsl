use core::fmt;
use std::{collections::HashMap, io::{self, Write}, rc::Rc};

mod primitives;
mod parsing;
mod evaluation;

use evaluation::evaluate;
use parsing::{parse, tokenize};
use primitives::{add, neg};

type Num = f64;

#[derive(Debug, Clone)]
enum LinslExpr {
    Bool(bool),
    Closure(Rc<LinslExpr>, Rc<LinslExpr>),
    List(Vec<LinslExpr>),
    Number(Num),
    Primitive(fn(&[LinslExpr]) -> Result<LinslExpr, LinslErr>),
    Symbol(String),
}

impl fmt::Display for LinslExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            LinslExpr::Bool(b) => if *b {"#t".to_string()} else {"#f".to_string()}
            LinslExpr::Closure(params, body) => format!("(lambda {}, {})", params, body),
            LinslExpr::Primitive(_) => "Primitive {}".to_string(),
            LinslExpr::List(xs) => {
                let strs : Vec<String> = xs
                    .iter()
                    .map(|x| x.to_string())
                    .collect();
                format!("({})", strs.join(" "))
            }
            LinslExpr::Number(v) => v.to_string(),
            LinslExpr::Symbol(s) => s.clone(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug)]
enum LinslErr {
    InternalError(String),
    ListError(String),
    PrimitivesError(String),
    SyntaxError(u32, u32),
    UnbalancedParens(u32, u32),
}

#[derive(Debug, Clone)]
struct LinslEnv {
    env: HashMap<String, LinslExpr>,
}

impl LinslEnv {
    fn default() -> LinslEnv {
        let mut env = HashMap::new();

        env.insert("+".to_string(), LinslExpr::Primitive(add));
        env.insert("-".to_string(), LinslExpr::Primitive(neg));
        LinslEnv { env }
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
            Err(e) => println!("{:?}", e),
        }
    }
    
}

#[cfg(test)]
mod tests {}
