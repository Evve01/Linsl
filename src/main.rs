//! A simple interpreter for a lisp/scheme like language

use std::io::{self, Write};

mod datatypes;
mod evaluation;
mod primitives;
mod parsing;

use evaluation::evaluate;
use parsing::{parse, tokenize};
use datatypes::{LinslEnv, LinslErr, LinslExpr, LinslRes};


fn parse_eval(expr: String, env: &mut LinslEnv) -> LinslRes {
    let (parse_res, rest) = parse(&tokenize(expr)?, 0)?;
    if !rest.is_empty() {
        return Err(LinslErr::SyntaxError("Unexpected characters at end.".to_string(), Vec::new()));
    };

    let res = evaluate(&parse_res, 0, env)?;
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
