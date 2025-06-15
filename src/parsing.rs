use crate::datatypes::{LinslErr, LinslExpr, Num, PosNum};

type InterParse = (LinslExpr, Vec<String>);

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

pub fn tokenize(expr: String) -> Result<Vec<String>, LinslErr> {
    if let Some((opening, closing)) = check_parens(&expr) {
        return Err(LinslErr::UnbalancedParens(opening, closing));
    };
    Ok (
        expr
            .replace("(", " ( ")
            .replace(")", " ) ")
            .split_whitespace()
            .map(|token| (token.to_string()))
            .collect()
    )
}

pub fn parse(tokens: &[String], start_pos: PosNum) -> Result<InterParse, LinslErr> {
    if tokens.is_empty() {
        return Err(LinslErr::InternalError("Unexpected EOF".to_string()));
    }

    let (token, rest) = tokens.split_first()
        .ok_or(
            LinslErr::InternalError("Could not read token".to_string())
        )?;

    match &token[..] {
        "(" => handle_result(read_list(rest), start_pos),
        ")" => Err(
            LinslErr::SyntaxError(
                "Unexpected closing parenthesis.".to_string(),
                vec![start_pos]
            )
        ),
        _ => Ok((parse_atom(token), rest.to_vec())),
    }
}

fn read_list(tokens: &[String]) -> Result<InterParse, LinslErr> {
    if tokens.is_empty() {
        return Err(
            LinslErr::SyntaxError(
                "Found only opening parentheses.".to_string(),
                vec![0]
            )
        );
    };

    let mut current_pos = 0;
    let mut elems : Vec<LinslExpr> = Vec::new();
    let mut toks = tokens.to_vec();
    loop {
        let (token, rest) = toks
            .split_first()
            .ok_or(
                LinslErr::SyntaxError(
                    "Could not read element of list.".to_string(),
                    vec![current_pos]
                )
        )?;

        if &token[..] == ")" {
            return Ok((LinslExpr::List(elems), rest.to_vec()));
        };

        let (exp, rem_toks) = parse(&toks, current_pos)?;

        elems.push(exp);
        current_pos += 1;
        toks = rem_toks;
    }
}

fn parse_atom(atom : &String) -> LinslExpr {
    match atom.as_ref() {
        "#t" => LinslExpr::Bool(true),
        "#f" => LinslExpr::Bool(false),
        _ => {
            let attempted_num : Result<Num, _> = atom.parse();
            match attempted_num {
                Ok(v) => LinslExpr::Number(v),
                Err(_) => LinslExpr::Symbol(atom.clone()),
            }
        }
    }
}

pub fn parse_num(expr: &LinslExpr, position: PosNum) -> Result<Num, LinslErr> {
    match expr {
        LinslExpr::Number(v) => Ok(*v),
        _ => Err(
            LinslErr::SyntaxError(
                format!("Expected numbers, found \'{}\'", expr), 
                vec![position]
            )
        ),
    }
}

pub fn parse_list_of_nums(nums: &[LinslExpr], start_index: PosNum) -> Result<Vec<Num>, LinslErr>{
    nums.iter()
        .zip(start_index..)
        .map(|(e, i)| parse_num(e, i))
        .collect::<Result<Vec<Num>, LinslErr>>()
}

pub fn parse_list_of_symbols(symbs: &LinslExpr) -> Result<Vec<String>, LinslErr> {
    let list = match symbs {
        LinslExpr::List(s) => Ok(s.clone()),
        _ => Err(
            LinslErr::SyntaxError(
                format!("Expected list of symbols, found \'{}\'", symbs),
                vec![0]
            )
        ),
    }?;

    list.iter().zip(0 as PosNum..).map(
        |(x, i)| {
            match x {
                LinslExpr::Symbol(s) => Ok(s.clone()),
                _ => Err(
                    LinslErr::SyntaxError(
                        format!("Expected symbol, found \'{}\'", x),
                        vec![i]
                    )
                ),
            }
        }
    ).collect()
}
