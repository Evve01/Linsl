use std::rc::Rc;

use crate::{LinslExpr, LinslErr, Num};

type TokensWithLoc = Vec<(String, (u32, u32))>;
type InterParse = ((LinslExpr, (u32, u32)), TokensWithLoc);

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

pub fn tokenize(expr: String, num_of_expr: u32) -> Result<TokensWithLoc, LinslErr> {
    if let Some((opening, closing)) = check_parens(&expr) {
        return Err(LinslErr::UnbalancedParens(opening as u32, closing as u32));
    };
    Ok (
        expr
            .replace("(", " ( ")
            .replace(")", " ) ")
            .split_whitespace()
            .zip(0_u32..)
            .map(|(token, pos)| (token.to_string(), (num_of_expr, pos)))
            .collect()
    )
}

pub fn parse(tokens: &[(String, (u32, u32))]) -> Result<InterParse, LinslErr> {
    if tokens.is_empty() {
        return Err(LinslErr::InternalError("Unexpected EOF".to_string()));
    }
    let (token, rest) = tokens.split_first()
        .ok_or(
            LinslErr::InternalError("Could not read token".to_string())
        )?;
    match &token.0[..] {
        "(" => read_list(rest),
        ")" => Err(LinslErr::SyntaxError(token.1.0, token.1.1)),
        _ => Ok((parse_atom(token), rest.to_vec())),
    }
}

fn read_list(tokens: &[(String, (u32, u32))]) -> Result<InterParse, LinslErr> {
    if tokens.is_empty() {
        return Err(LinslErr::ListError("Found only opening parentheses.".to_string()));
    };

    let (start_x, start_y): (u32, u32) = tokens[0].1;
    let mut elems : Vec<LinslExpr> = Vec::new();
    let mut toks = tokens.to_vec();
    loop {
        let (token, rest) = toks
            .split_first()
            .ok_or(
                LinslErr::InternalError(
                    format!("Could not read element of list at ({}, {})", start_x, start_y)
                )
        )?;

        if &token.0[..] == ")" {
            return Ok(((LinslExpr::List(elems), (start_x, start_y)), rest.to_vec()));
        }

        let (exp, rem_toks) = parse(&toks)?;
        elems.push(exp);
        toks = rem_toks;
    }
}

fn parse_atom(atom : &(String, (u32, u32))) -> (LinslExpr, (u32, u32)) {
    match atom.0.as_ref() {
        "#t" => (LinslExpr::Bool(true), atom.1),
        "#f" => (LinslExpr::Bool(false), atom.1),
        _ => {
            let attempted_num : Result<Num, _> = atom.0.parse();
            match attempted_num {
                Ok(v) => (LinslExpr::Number(v), atom.1),
                Err(_) => (LinslExpr::Symbol(atom.0.clone()), atom.1)
            }
        }
    }
}

pub fn parse_num(expr: &LinslExpr) -> Result<Num, LinslErr> {
    match expr {
        LinslExpr::Number(v) => Ok(*v),
        _ => Err(LinslErr::PrimitivesError(format!("Expected numbers, found \'{}\'", expr))),
    }
}

pub fn parse_list_of_nums(nums: &[LinslExpr]) -> Result<Vec<Num>, LinslErr>{
    nums.iter()
        .map(parse_num)
        .collect::<Result<Vec<Num>, LinslErr>>()
}

pub fn parse_list_of_symbols(symbs: Rc<LinslExpr>) -> Result<Vec<String>, LinslErr> {
    let list = match symbs.as_ref() {
        LinslExpr::List(s) => Ok(s.clone()),
        _ => Err(LinslErr::PrimitivesError(
                format!("Expected list of symbols, found \'{}\'", symbs)
            )
        ),
    }?;

    list.iter().map(
        |x| {
            match x {
                LinslExpr::Symbol(s) => Ok(s.clone()),
                _ => Err(
                    LinslErr::PrimitivesError(
                        format!("Expected symbol, found \'{}\'", x)
                    )
                ),
            }
        }
    ).collect()
}
