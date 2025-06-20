//! The datatypes used throughout the code base.

use std::{collections::{HashMap, VecDeque}, fmt, io::BufRead};

use regex::Regex;

use crate::{primitives::{add, car, cdr, eq, gr, inv, mul, neg}};

pub type Num = f64;
pub type PosNum = usize;
/// Positions of expressions are tracked and used only to specify where a syntax error has
/// occurred.
pub type Pos = Vec<PosNum>;

pub type LinslRes = Result<LinslExpr, LinslErr>;

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
    Primitive(fn(&[LinslExpr]) -> LinslRes),
    Symbol(String),
    /// A macro, which is similar to a closure but does not evaluate its parameters.
    Macro(Box<LinslExpr>, Box<LinslExpr>),
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
            LinslExpr::Macro(ps, bd)    => format!("(macro {}, {})", ps, bd),
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
    
    pub fn new<'a>(outer: &'a LinslEnv) -> LinslEnv<'a> {
        LinslEnv { 
            inner: HashMap::new(),
            outer: Some(outer)
        }
    }
}

/// Tokenizer, resopnisble for the retrieval and tokenization of input strings.
/// `inputs` are the sources to read from, such as files or stdin.
/// tokens holds the results of tokenizing a single line.
///
/// When created must be supplied with inputs. It will then tokenize the first line from the fisrt
/// input. Tokens can be retrieved using the next_token method.
pub struct Tokenizer {
    /// The inputs to read from, in order.
    inputs: VecDeque<Box<dyn BufRead>>,
    /// Tokens from the latest tokenized line.
    tokens: VecDeque<String>,
}

impl Tokenizer {
    /// Create a new Tokenizer, which will read code from the given inputs.
    pub fn new(inputs: VecDeque<Box<dyn BufRead>>) -> Result<Self, LinslErr> {
        let mut tokenizer = Self {
            inputs,
            tokens: VecDeque::new(),
        };

        tokenizer.tokenize_line()?;
        Ok(tokenizer)
    }

    /// Add a new input for the tokenizer to read from. Will be read from when all previously added
    /// inputs are exhausted.
    pub fn add_input(&mut self, input: Box<dyn BufRead>) -> () {
        self.inputs.push_back(input);
    }

    /// Returns the next token, or None if all inputs have been exhausted.
    pub fn next_token(&mut self) -> Result<Option<String>, LinslErr> {
        // First, try to get the next token from the tokens.
        let token = self.tokens.pop_front();

        // If the tokens VecDeque was not empty, return the popped element,
        if token.is_some() {
            Ok(token)
        }
        // else tokenize the next line and return the first token from that line.
        else {
            self.tokenize_line()?;
            Ok(self.tokens.pop_front())
        }
    }

    fn regex() -> Regex {
        Regex::new(r"\s*(,@|[('`,)]|;.*|[^\s('`,;)]*)(.*)").unwrap()
    }

    /// Finds the next line and tokenizes it. If no more valid input exists returns None.
    fn tokenize_line(&mut self) -> Result<Option<()>, LinslErr> {
        // First, let's try to get the next line from the inputs.
        let line = self.get_line();

        // If something went wrong, we return an error.
        if let Err(e) = line {
            return Err(LinslErr::InternalError(format!("{:?}", e)));
        }
        // Otherwise, we check if we ran out of input.
        let initial_rest = match line.unwrap() {
            // If there was input, that input is fine as is,
            Some(s) => s.to_string(),
            // otherwise we return None, marking EOF for all inputs.
            None => return Ok(None),
        };

        // At this point, we know that initial_rest contains a line of text. We can therefore begin
        // tokenizing it.
        // At the start of tokenization, the entire line is "the rest".
        let mut rest: &str = &initial_rest;
        let re = Tokenizer::regex();

        // We then start tokenizing, and keep doing that until we've tokenized everything.
        loop {
            for (_, [result, unmatched]) in re.captures_iter(rest).map(|c| c.extract()) {
                // If the token is not a comment, add it to tokens.
                if result.chars().nth(0) != Some(';') {
                    self.tokens.push_back(result.to_string());
                }
                rest = unmatched;
            };

            if rest.is_empty() {
                break;
            }
        };
        Ok(Some(()))
    }

    /// Gets the next line from the inputs. If the current head of the inputs is empty, will pop it
    /// and start reading from the next. If there is no valid input left, returns None.
    fn get_line(&mut self) -> Result<Option<String>, std::io::Error> {
        // First, check if there exists an input to get text from.
        // If not, return none.
        if self.inputs.front().is_none() {
            return Ok(None);
        };
        // If it does exist, check if it is empty.
        if self.inputs[0].fill_buf()?.is_empty() {
            // If it is, remove that input and continue to the next one.
            self.inputs.pop_front();
            return self.get_line();
        };

        // With these checks done, we know that a non-empty input exists.
        // Therefore, we create a string to read the line into,
        let mut initial_rest = String::new();
        // fill it,
        let _ = self.inputs[0].read_line(&mut initial_rest);
        // and return it.
        Ok(Some(initial_rest))
    }
}
