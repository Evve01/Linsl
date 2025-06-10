use crate::{LinslExpr, LinslErr, Num};

type TokensWithLoc = Vec<(String, (u32, u32))>;
type InterParse = (LinslExpr, TokensWithLoc);

pub fn tokenize(expr: String, num_of_expr: u32) -> TokensWithLoc {
    expr
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .zip(0_u32..)
        .map(|(token, pos)| (token.to_string(), (num_of_expr, pos)))
        .collect()
}

pub fn parse(tokens: &[(String, (u32, u32))]) -> Result<InterParse, LinslErr> {
    let (token, rest) = tokens.split_first()
        .ok_or(
            LinslErr::InternalError("Could not read Token".to_string())
        )?;
    if tokens.is_empty() {
        return Err(LinslErr::InternalError("Unexpected EOF".to_string()));
    }
    match &token.0[..] {
        "(" => read_list(rest),
        ")" => Err(LinslErr::SyntaxError(token.1.0, token.1.1)),
        _ => Ok((parse_atom(token), rest.to_vec())),
    }
}

fn read_list(tokens: &[(String, (u32, u32))]) -> Result<InterParse, LinslErr> {
    let starting_index : (u32, u32) = tokens[0].1;
    let mut elems : Vec<LinslExpr> = Vec::new();
    let mut toks = tokens.to_vec();
    loop {
        let (token, rest) = toks
            .split_first()
            .ok_or(LinslErr::UnbalancedParens(starting_index.0, starting_index.1))
            ?;

        if &token.0[..] == ")" {
            return Ok((LinslExpr::List(elems), rest.to_vec()));
        }

        let (exp, rem_toks) = parse(&toks)?;
        elems.push(exp);
        toks = rem_toks;
    }
}

fn parse_atom(atom : &(String, (u32, u32))) -> LinslExpr {
    let attempted_num : Result<Num, _> = atom.0.parse();
    match attempted_num {
        Ok(v) => LinslExpr::Number(v),
        Err(_) => LinslExpr::Symbol(atom.0.clone())
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
