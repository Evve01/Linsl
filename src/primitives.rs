//! The built in functions/forms. Here we define precisely as much as we need to to be able to
//! define any other functions/macros we desire in Linsl code.

use crate::{LinslExpr, LinslErr, Num};
use crate::parsing::{parse_num, parse_list_of_nums};

/// Compute the sum of a list of (numeric) arguments.
pub fn add(exprs: &[LinslExpr]) -> Result<LinslExpr, LinslErr>{
    let sum = parse_list_of_nums(exprs)?.iter().fold(0 as Num, |sum, v| sum + v);
    Ok(LinslExpr::Number(sum))
}

/// Negate a single element.
pub fn neg(expr: &[LinslExpr]) -> Result<LinslExpr, LinslErr>{
    let mut num : Num = 0 as Num;
    if !expr.is_empty() {
        num = parse_num(&expr[0])?;
    }
    Ok(LinslExpr::Number(-num))
}

/// Compute the product of a list of (numeric) arguments.
pub fn mul(exprs: &[LinslExpr]) -> Result<LinslExpr, LinslErr> {
    let mul = parse_list_of_nums(exprs)?.iter().fold(1 as Num, |mul, v| mul * v);
    Ok(LinslExpr::Number(mul))
}

pub fn inv(expr: &[LinslExpr]) -> Result<LinslExpr, LinslErr> {
    let mut num: Num = 1 as Num;

    if !expr.is_empty() {
        num = parse_num(&expr[0])?;
    } else {
        return Err(LinslErr::PrimitivesError("No number to invert!".to_string()));
    }

    if num == 0 as Num {
        return Err(LinslErr::PrimitivesError("Cannot invert 0".to_string()));
    }

    Ok(LinslExpr::Number(1 as Num/num))
}
