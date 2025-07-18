//! The built in functions/forms. Here we define precisely as much as we need to to be able to
//! define any other functions/macros we desire in Linsl code.

use crate::datatypes::{LinslRes, Num};
use crate::{LinslExpr, LinslErr};
use crate::parsing::{parse_list_of_nums, parse_num};

/// Compute the sum of a list of (numeric) arguments.
pub fn add(exprs: &[LinslExpr]) -> LinslRes {
    let sum = parse_list_of_nums(exprs.into())?.iter().fold(0 as Num, |sum, v| sum + v);
    Ok(LinslExpr::Number(sum))
}

/// Combine supplied lists to one list, in the order they appear. That is,
/// (append '(1) '(2) '(3)) becomes (1 2 3).
///
/// If only supplied with a single list, return that list.
pub fn append(exprs: &[LinslExpr]) -> LinslRes {
    // First, ensure that arguments were supplied.
    if exprs.is_empty() {
        return Err(
            LinslErr::SyntaxError(
                // TODO: Fix pos
                "append needs at least one argument, none were supplied".to_string(),
                (0, 0)
            )
        );
    };

    // If only a single argument was supplied, ensure it is a list and then return it.
    if exprs.len() == 1 {
        match &exprs[0] {
            LinslExpr::List(linsl_exprs) => Ok(
                LinslExpr::List(linsl_exprs.clone())
            ),
            _ => Err(
                LinslErr::SyntaxError(
                    "append must be supplied with list(s)".to_string(),
                    (0, 0)
                )
            ),
        }
    } else {
        // Since we know that there are at least two arguments we create a vector to store all the
        // arguments in..
        let mut vec: Vec<LinslExpr> = Vec::new();
        let mut pos: usize = 0;
        // We then iterate over the arguments
        while pos < exprs.len() {
            // extracting their elements
            if let LinslExpr::List(mut linsl_exprs) = exprs[pos].clone() {
                // and add those to the vector created above.
                vec.append(&mut linsl_exprs);
            } else {
                // If a non-list argument is encountered, return an error.
                return Err(
                    LinslErr::SyntaxError(
                        "append must be supplied with list(s)".to_string(),
                        (0, 0)
                    )
                )
            };
            pos += 1;
        };
        // Finally, return a new list with all the elements from the lists supplied.
        Ok(LinslExpr::List(vec))
    }
}

/// Return the first element of a list.
pub fn car(expr: &[LinslExpr]) -> LinslRes {
    if expr.len() != 1 {
        return Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                format!("Expected 1 argument, found {}", expr.len()),
                (0, 0)
            )
        );
    };

    match &expr[0] {
        LinslExpr::List(linsl_exprs) => match linsl_exprs.first() {
            Some(e) => Ok(e.clone()),
            None => Ok(LinslExpr::List(Vec::new())),
        }
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "Can only find car of lists".to_string(),
                (0, 0)
            )
        )
    }
}

/// Return the tail of a list.
pub fn cdr(expr: &[LinslExpr]) -> LinslRes {
    if expr.len() != 1 {
        return Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                format!("Expected 1 argument, found {}", expr.len()),
                (0, 0)
            )
        );
    };

    match &expr[0] {
        LinslExpr::List(linsl_exprs) => {
            match linsl_exprs.clone().split_first() {
                Some((_, tail)) => Ok(LinslExpr::List(tail.to_vec())),
                None => Ok(LinslExpr::List(Vec::new())),
            }
        },
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "Can only find cdr of lists.".to_string(),
                (0, 0)
            )
        )
    }
}

/// Compare two numbers, symbols or booleans for equality.
pub fn eq(exprs: &[LinslExpr]) -> LinslRes {
    if exprs.len() != 2 {
        return Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                format!("Expected 2 arguments to compare, got {}", exprs.len()),
                (0, 0)
            )
        );
    };

    let res: bool = match (exprs[0].clone(), exprs[1].clone()) {
        (LinslExpr::Bool(b1), LinslExpr::Bool(b2)) => b1 == b2,
        (LinslExpr::Number(v1), LinslExpr::Number(v2)) => v1 == v2, 
        (LinslExpr::Symbol(s1), LinslExpr::Symbol(s2)) => s1 == s2,
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "Can only compare expressions the same types, and only bools, numbers and symbols."
                    .to_string(),
                    (0, 0)
            )
        )?,
    };

    Ok(LinslExpr::Bool(res))
}

/// Compare two numbers to see if the first is greater than the second.
pub fn gr(exprs: &[LinslExpr]) -> LinslRes {
    if exprs.len() != 2 {
        return Err(
            LinslErr::InternalError(
                format!("Expected two numbers two compare, got {}", exprs.len())
            )
        )
    };

    let res: bool = match (exprs[0].clone(), exprs[1].clone()) {
        (LinslExpr::Number(v1), LinslExpr::Number(v2)) => v1 > v2,
        _ => Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "Can only compare numbers with >".to_string(),
                (0, 0)
            )
        )?,
    };

    Ok(LinslExpr::Bool(res))
}

/// Compute the multiplicative inverse of a (numeric) argument.
pub fn inv(expr: &[LinslExpr]) -> LinslRes {
    if expr.is_empty() {
        return Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "No number to invert!".to_string(),
                (0, 0)
            )
        );
    };

    let num = parse_num(&expr[0])?;

    if num == 0 as Num {
        return Err(
            // TODO: Fix pos.
            LinslErr::SyntaxError(
                "Cannot invert 0".to_string(),
                (0, 0)
            )
        );
    };

    Ok(LinslExpr::Number(1 as Num/num))
}

/// Take an arbitrary number of elements, and return a list containing those elements. For example,
/// (list 1 + 2) becomes (1 + 2), and (list) becomes ().
pub fn list(exprs: &[LinslExpr]) -> LinslRes {
    Ok(LinslExpr::List(exprs.to_vec()))
}

/// Compute the product of a list of (numeric) arguments.
pub fn mul(exprs: &[LinslExpr]) -> LinslRes {
    let mul = parse_list_of_nums(exprs.into())?.iter().fold(1 as Num, |mul, v| mul * v);
    Ok(LinslExpr::Number(mul))
}

/// Negate a single element.
pub fn neg(expr: &[LinslExpr]) -> LinslRes {
    let mut num : Num = 0 as Num;
    if !expr.is_empty() {
        num = parse_num(&expr[0])?;
    }
    Ok(LinslExpr::Number(-num))
}

pub fn is_nil(expr: &[LinslExpr]) -> LinslRes {
    if expr.len() != 1 {
        return Err(
            LinslErr::SyntaxError(
                // TODO: Fix pos.
                format!("Expected 1 argument, found {}", expr.len()),
                (0, 0)
            )
        );
    };

    match &expr[0] {
        LinslExpr::List(linsl_exprs) => Ok(
            LinslExpr::Bool(linsl_exprs.is_empty())
        ),
        _ => Ok(
            LinslExpr::Bool(false)
        )
    }
}

pub fn eq_types(exprs: &[LinslExpr]) -> LinslRes {
    if exprs.len() != 2 {
        return Err(
            LinslErr::SyntaxError(
                // TODO: Fix pos.
                format!("Expected 2 arguments, found {}", exprs.len()),
                (0, 0)
            )
        );
    };
    let (a, b) = (exprs[0].clone(), exprs[1].clone());

    let bool = matches!((a, b), 
        (LinslExpr::Bool(_), LinslExpr::Bool(_))
        | (LinslExpr::Closure(_, _), LinslExpr::Closure(_, _))
        | (LinslExpr::List(_), LinslExpr::List(_))
        | (LinslExpr::Number(_), LinslExpr::Number(_))
        | (LinslExpr::Primitive(_), LinslExpr::Primitive(_))
        | (LinslExpr::Symbol(_), LinslExpr::Symbol(_))
        | (LinslExpr::Macro(_, _), LinslExpr::Macro(_, _))
    );

    Ok(LinslExpr::Bool(bool))
}
