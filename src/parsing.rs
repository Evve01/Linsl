use std::{collections::VecDeque, io::BufRead};

use regex::Regex;

use crate::datatypes::{LinslErr, LinslExpr, Num, Pos, PosNum};

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
    tokens: VecDeque<(String, Pos)>,
    /// Location of the latest delivered token, to be used when reporting errors.
    latest_pos: Pos,
}

impl Tokenizer {
    /// Create a new Tokenizer, which will read code from the given inputs.
    pub fn new(inputs: VecDeque<Box<dyn BufRead>>) -> Result<Self, LinslErr> {
        let mut tokenizer = Self {
            inputs,
            tokens: VecDeque::new(),
            latest_pos: (0, 0)
        };

        tokenizer.tokenize_line()?;
        Ok(tokenizer)
    }

    /// Add a new input for the tokenizer to read from. Will be read from when all previously added
    /// inputs are exhausted.
    pub fn add_input(&mut self, input: Box<dyn BufRead>) {
        self.inputs.push_back(input);
    }

    /// Returns the next token, or None if all inputs have been exhausted.
    pub fn next_token(&mut self) -> Result<Option<String>, LinslErr> {
        // First, try to get the next token from the tokens.
        // If there is one, update the latest position and return the token.
        if let Some((token, pos)) = self.tokens.pop_front() {
            self.latest_pos = pos;
            Ok(Some(token))
        }
        else {
        // else tokenize the next line.
            self.tokenize_line()?;
            match self.tokens.pop_front() {
                // If that returns a new token, do same as above.
                Some((t, p)) => {
                    self.latest_pos = p;
                    Ok(Some(t))
                },
                // If it doesn't, we're out of input; we signal this by returning None.
                None => Ok(None),
            }
        }
    }

    /// Get the next token without popping it from the tokens stream. It is used for example when
    /// parsing lists; a list needs to check if the next token is a closing parenthesis to know
    /// if the list has ended. If it hasn't it needs to call `parse`, and parse the next token. If
    /// using `next_token` this token would already have disappeared, thus the need for this
    /// function.
    pub fn peek(&mut self) -> Option<String> {
        // If there are parsed tokens left, return a copy of the first one.
        if self.tokens.len() > 0 {
            return Some(self.tokens[0].0.clone());
        };

        // If not, attempt to parse more.
        self.tokenize_line();
        // If there are new tokens at this point, return a copy of the first one,
        if self.tokens.len() > 0 {
            Some(self.tokens[0].0.clone())
        } else {
            // if not we are out of input.
            None
        }
    }

    /// Returns the position of the latest retrieved token.
    pub fn get_pos(&self) -> Pos {
        self.latest_pos.clone()
    }

    /// Regex used for getting tokens.
    fn regex() -> Regex {
        Regex::new(r"\s*(,@|[('`,)]|;.*|[^\s('`,;)]*)(.*)").unwrap()
    }

    /// Finds the next line and tokenizes it. If no more valid input exists returns None.
    fn tokenize_line(&mut self) -> Result<Option<()>, LinslErr> {
        // First, let's try to get the next line from the inputs.
        let (line, line_num) = match self.get_line(self.latest_pos.0) {
            // If possible, we simply unwrap the line and the line number.
            Ok(Some((s, l))) => (s, l),
            // If we've run out of input, we simply return None.
            Ok(None) => {
                return Ok(None);
            },
            // If something went wrong, we return an error.
            Err(e) => {
                return Err(LinslErr::InternalError(format!("{:?}", e)));
            }
        };

        // At this point, we know that line contains a line of text. We can therefore begin
        // tokenizing it.
        // At the start of tokenization, the entire line is "the rest", and the position is the
        // start of the line.
        let mut rest: &str = &line;
        let mut col: PosNum = 0;
        let re = Tokenizer::regex();

        // We then start the actual tokenization, and keep doing that until we've tokenized
        // everything.
        loop {
            for (_, [result, unmatched]) in re.captures_iter(rest).map(|c| c.extract()) {
                // If the token is non-empty and not a comment, add it to tokens.
                if !result.is_empty() && result.chars().nth(0) != Some(';') {
                    self.tokens.push_back((result.to_string(), (line_num, col)));
                    // Then increment the column number so it points to after the read token.
                    col += result.len() + 1;
                }
                rest = unmatched;
            };

            if rest.is_empty() {
                break;
            }
        };
        Ok(Some(()))
    }

    /// Gets the next line from the inputs, along with the corresponding line number. If the
    /// current head of the inputs is empty, will pop it and start reading from the next. If there
    /// is no valid input left, returns None.
    fn get_line(&mut self, last_line: PosNum) -> Result<Option<(String, PosNum)>, std::io::Error> {
        // First, check if there exists an input to get text from.
        // If not, return none.
        if self.inputs.front().is_none() {
            return Ok(None);
        };
        // If it does exist, check if it is empty.
        if self.inputs[0].fill_buf()?.is_empty() {
            // If it is, remove that input and continue to the next one.
            self.inputs.pop_front();
            // Since we've started on a new input, we reset the position.
            return self.get_line(0);
        };

        // With these checks done, we know that a non-empty input exists.
        // Therefore, we create a string to read the line into,
        let mut initial_rest = String::new();
        // fill it,
        let _ = self.inputs[0].read_line(&mut initial_rest);
        // remove the \n character at the end (otherwise the regex won't work),
        initial_rest.pop();
        // and return it.
        Ok(
            Some(
                (initial_rest, 
                 if last_line == 0 {0} else {last_line + 1})
            )
        )
    }
}

/// Checks if there are as many opening as closing parentheses.
/// If not, returns the number of parentheses found.
/// Else, returns None.
fn check_parens(string: &str) -> Option<(usize, usize)> {
    let opening = string.matches("(").count();
    let closing = string.matches(")").count();
    
    if opening != closing {
        Some((opening, closing))
    } else {
        None
    }
}

/*
pub fn handle_result<T>(res: Result<T, LinslErr>, pos: PosNum) -> Result<T, LinslErr> {
    match res {
        Ok(_) => res,
        Err(e) => match e {
            LinslErr::InternalError(_) => Err(e),
            LinslErr::UnbalancedParens(_, _) => Err(e),
            LinslErr::SyntaxError(s, items) => {
                let mut new_pos = items.clone();
                new_pos.push(pos);
                Err(LinslErr::SyntaxError(s, new_pos))
            }
        }
    }
}
*/

pub fn parse(mut tokenizer: &mut Tokenizer) -> Result<LinslExpr, LinslErr> {
    // We begin by retrieving the next token, if any.
    let token = match tokenizer.next_token() {
        Ok(Some(t)) => t,
        // If no token exists, something had gone wrong; this function is only called when there
        // needs to be more tokens in order to form a valid expression.
        Ok(None) => {
            return Err(LinslErr::InternalError("Unexpected EOF.".to_string()));
        },
        // If something went wrong in the tokenizer, we simply return that error.
        Err(e) => {
            return Err(e);
        },
    };

    // We then check the token.
    match token.as_str() {
        // If it is a quote or quasiquote, parse the rest accordingly.
        "'" => parse_quote(tokenizer),
        "`" => parse_quasiquote(&mut tokenizer),
        // An opening parenthesis means we start reading a new list.
        "(" => parse_list(tokenizer, parse),
        // If we encounter a closing parenthesis something went wrong.
        ")" => Err(
            LinslErr::SyntaxError(
                "Unexpected closing parenthesis.".to_string(),
                tokenizer.get_pos()
            )
        ),
        // Otherwise we attempt to parse it as an atom.
        _ => Ok(parse_atom(&token)),
    }
}

/// This function is called during parsing, if we encounter anything which is not a list or a
/// quoted expression.
fn parse_atom(atom : &str) -> LinslExpr {
    match atom {
        // If the atom is `#t` or `#f` we can instanly handle it.
        "#t" => LinslExpr::Bool(true),
        "#f" => LinslExpr::Bool(false),
        // If it is not, we check if it is a number; if it is then good, otherwise we treat it as a
        // symbol. We DO NOT check if it is defined etc here, that is done during evaluation.
        _ => {
            let attempted_num : Result<Num, _> = atom.parse();
            match attempted_num {
                Ok(v) => LinslExpr::Number(v),
                Err(_) => LinslExpr::Symbol(atom.to_string()),
            }
        }
    }
}

/// If an opening parenthesis is encountered, this function is called. It parses -- using the
/// supplied parser function -- until it encounters a closing parenthesis.
fn parse_list(tokenizer: &mut Tokenizer, parser: fn(&mut Tokenizer) -> Result<LinslExpr, LinslErr>) -> Result<LinslExpr, LinslErr> {
    // FIrst, create a vec to keep the list elements in.
    let mut list_elems: Vec<LinslExpr> = Vec::new();
    // Then we start looping over tokens:
    loop {
        // Retrieve the next token, if one is available
        let token = match tokenizer.peek() {
            Some(t) => t,
            None => {
                return Err(
                    LinslErr::SyntaxError(
                        "Found only opening parentheses.".to_string(),
                        tokenizer.get_pos()
                    )
                );
            },
        };

        // If the token is `)` the list has ended,
        if token == ")" {
            // so we remove the closing parenthesis
            let _ = tokenizer.next_token();
            // and stop looping.
            break;
        };

        // Otherwise, parse the next expresion, and 
        let exp = parser(tokenizer)?;
        // add it as an element to the list.
        list_elems.push(exp);
    };

    // When done looping, return a list expression.
    Ok(LinslExpr::List(list_elems))
}

pub fn parse_list_of_nums(nums: Box<[LinslExpr]>) -> Result<Vec<Num>, LinslErr>{
    (*nums).iter()
        .map(|e| parse_num(e))
        .collect::<Result<Vec<Num>, LinslErr>>()
}

pub fn parse_list_of_symbols(symbs: &LinslExpr) -> Result<Vec<String>, LinslErr> {
    let list = match symbs {
        LinslExpr::List(s) => Ok(s.clone()),
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                format!("Expected list of symbols, found \'{}\'", symbs),
                (0, 0)
            )
        ),
    }?;

    list.iter().map(
        |x| {
            match x {
                LinslExpr::Symbol(s) => Ok(s.clone()),
                _ => Err(
                    // TODO: Fix pos.
                    LinslErr::SyntaxError(
                        format!("Expected symbol, found \'{}\'", x),
                        (0, 0)
                    )
                ),
            }
        }
    ).collect()
}

pub fn parse_num(expr: &LinslExpr) -> Result<Num, LinslErr> {
    match expr {
        LinslExpr::Number(v) => Ok(*v),
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                format!("Expected numbers, found \'{}\'", expr), 
                (0, 0)
            )
        ),
    }
}

/// Rewrites 'x -- where x is any Linsl expression -- as (quote x).
fn parse_quote(tokenizer: &mut Tokenizer) -> Result<LinslExpr, LinslErr> {
    // Get the next expression.
    match parse(tokenizer) {
        // If this went well, wrap it in a list with head quote,
        Ok(expr) => Ok(
            LinslExpr::List(
                vec![LinslExpr::Symbol("quote".to_string()),
                    expr]
            )
        ),
        // else, return the error.
        Err(e) => Err(e),
    }
}

/// Rewrites 
/// 1. `(x_1 ... ,x_n ... x_m) as (append (list (quote x_1)) ... (list x_n) ... (list (quote x_m))),
/// 2. `,x as x 
/// 3. `(x_1 ... ,@x ... x_n) as (append (list (quote x_1)) ... x ... (list (quote x_m))) 
/// 4. `x as (quote x)
fn parse_quasiquote(tokenizer: &mut Tokenizer) -> Result<LinslExpr, LinslErr> {
    // We start by getting the first token:
    let token = match tokenizer.next_token()? {
        // Take the token if there was one, else return an error.
        Some(t) => t,
        None => return Err(
            LinslErr::SyntaxError(
                "Unexpected EOF.".to_string(),
                tokenizer.get_pos()
            )
        ),
    };

    // We can now inspect the token.
    match token.as_str() {
        // A comma escapes the quoting, i.e. `,x <=> x, and so we simply continue.
        "," => parse(tokenizer),
        // Here we do mutch the same as for a comma, but we need to add unwrap.
        ",@" => Err(
            LinslErr::SyntaxError(
                "Cannot have ,@ at top level of `".to_string(),
                tokenizer.get_pos()
            )
        ),
        // If we encounter an opening parenthesis -- which is what we often do when the user uses
        // quasiquotes -- we parse the list.
        "(" => {
            if let LinslExpr::List(mut v) = parse_list(tokenizer, parse_quasiquote_elem_in_list)? {
                v.insert(0, LinslExpr::Symbol("append".to_string()));
                Ok(LinslExpr::List(v))
            } else {
                panic!("parse_list did not return a list when parsing quasi-quote!")
            }
        }
        
        // If none of the others have matched, then we have a non-escaped atom; in this case a
        // quasiquote behaves the same as a regular quote.
        _ => parse_quote(tokenizer),
    }
}

/// Rewrites `(..) according to the rules described above.
fn parse_quasiquote_elem_in_list(tokenizer: &mut Tokenizer) -> Result<LinslExpr, LinslErr>{
    // First, get token as always,
    let token = match tokenizer.next_token()? {
        Some(t) => t,
        None => {
            return Err(
                LinslErr::SyntaxError(
                    "Unexpected EOF.".to_string(),
                    tokenizer.get_pos()
                )
            );
        }
    };

    // then check it.
    match token.as_str() {
        // ,x => (list x)
        "," => Ok(
            LinslExpr::List(vec![
                LinslExpr::Symbol("list".to_string()),
                parse(tokenizer)?
            ])
        ),
        // ,@x => x
        ",@" => Ok(
            parse(tokenizer)?
        ),
        // else: x => (list (quote x))
        _ => Ok(
            LinslExpr::List(vec![
                LinslExpr::Symbol("list".to_string()),
                LinslExpr::List(vec![
                    LinslExpr::Symbol("quote".to_string()),
                    parse(tokenizer)?
                ])
            ])
        ),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup(input: Box<dyn BufRead>) -> Tokenizer {
        let mut vec: VecDeque<Box<dyn BufRead>> = VecDeque::new();
        vec.push_front(Box::new(input));
        Tokenizer::new(vec).unwrap()
    }

    #[test]
    fn tokenize_line() {
        let s = "(+ 1 2 3)\n";
        let tokenizer = setup(Box::new(s.as_bytes()));

        let string: String = s.to_string();

        let mut tokens: Vec<String> = Vec::new();
        
        for s in string.chars() {
            if s != ' ' && s != '\n' {
                tokens.push(s.to_string());
            }
        }
        
        assert_eq!(tokens.len(), 6);

        let test = tokens
            .iter()
            .zip(&tokenizer.tokens)
            .fold(true, |acc, (c, t)| (*c == t.0) && acc);
        if !test {
            println!("{:?}", tokens);
            println!("{:?}", &tokenizer.tokens);
        };
        assert!(test);
    }

    #[test]
    fn check_peek_length() {
        let s = "(+ 1 2 3)\n";
        let mut tokenizer = setup(Box::new(s.as_bytes()));

        let len0 = tokenizer.tokens.len();
        let _ = tokenizer.peek();
        let len1 = tokenizer.tokens.len();
        assert_eq!(len0, len1)
    }

    #[test]
    fn check_peek_idempotence() {
        let s = "(+ 1 2 3)\n";
        let mut tokenizer = setup(Box::new(s.as_bytes()));

        let a = tokenizer.peek().unwrap();
        let b = tokenizer.peek().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn next_token() {
        let s = "(+ 1 2 3)\n";
        let mut tokenizer = setup(Box::new(s.as_bytes()));

        let string: Vec<String> = s
            .replace("(", " ( ")
            .replace(")", " ) ")
            .split_whitespace()
            .map(|c| c.to_string())
            .collect();

        (0..string.len()).for_each(|n| {
            let token = tokenizer.next_token().unwrap().unwrap();
            assert_eq!(string[n], token)
        });
    }
}
