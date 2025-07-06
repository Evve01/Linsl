//! A simple interpreter for a lisp/scheme like language

use std::{collections::VecDeque, env::args, fs::{self, File}, io::{self, BufRead, BufReader, Result, Write}};

mod datatypes;
mod evaluation;
mod primitives;
mod parsing;

use evaluation::evaluate;
use parsing::{parse,  Tokenizer};
use datatypes::{LinslEnv, LinslErr, LinslExpr, LinslRes};

fn parse_eval(tokenizer: &mut Tokenizer, env: &mut LinslEnv) -> LinslRes {
    let parse_res = parse(tokenizer)?;

    let res = evaluate(&parse_res, env)?;
    Ok(res)
}

/// Returns Stdin as an input source.
fn get_stdin() -> Box<dyn BufRead> {
    Box::new(io::stdin().lock())
}

/// Attempts to create an input source from a file.
/// Returns an error if the file does not exist, or cannot be interpreted as a UTF-8 string.
fn get_file(path: &str) -> Result<Box<dyn BufRead>> {
    match fs::read_to_string(path) {
        Ok(_) => Ok(Box::new(BufReader::new(File::open(path)?))),
        Err(e) => Err(e)
    }
}

/// Setup sources to read input from, and return them.
fn get_input() -> Result<VecDeque<Box<dyn BufRead>>> {
    // First, create vecdeque to store the inputs in.
    let mut vec: VecDeque<Box<dyn BufRead>> = VecDeque::new();

    // Then, check if any file paths were passed.
    let args = args().skip(1);
    if args.len() == 0 {
        // If not, then simply return the vec containing stdin.
        vec.push_back(get_stdin());
        Ok(vec)
    } else {
        // If there are file specified, try to add them instead.
        for path in args {
            let file = get_file(&path)?;
            vec.push_back(file);
        };
        Ok(vec)
    }

}

fn main() {
    let env = &mut LinslEnv::default();
    let mut tkzr = Tokenizer::new(get_input().unwrap()).unwrap();

    loop {
        print!("Linsl> ");
        io::stdout().flush().expect("Could not print!");
        
        match parse_eval(&mut tkzr, env) {
            Ok(res) => println!("{}", res),
            Err(e) => println!("{}", e),
        }
    }
    
}

#[cfg(test)]
mod tests {}
