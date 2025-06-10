use crate::{LinslExpr, LinslErr, Num};
use crate::parsing::{parse_num, parse_list_of_nums};


pub fn add(exprs: &[LinslExpr]) -> Result<LinslExpr, LinslErr>{
    let sum = parse_list_of_nums(exprs)?.iter().fold(0 as Num, |sum, v| sum + v);
    Ok(LinslExpr::Number(sum))
}

pub fn neg(expr: &[LinslExpr]) -> Result<LinslExpr, LinslErr>{
    let mut num : Num = 0 as Num;
    if !expr.is_empty() {
        num = parse_num(&expr[0])?;
    }
    Ok(LinslExpr::Number(-num))
}
