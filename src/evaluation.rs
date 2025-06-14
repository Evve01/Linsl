//! Code for evaluating Linsl expressions.

use std::{collections::HashMap, rc::Rc};

use crate::{datatypes::{LinslEnv, LinslErr, LinslExpr, PosNum}, parsing::{handle_result, parse_list_of_symbols}};

/// Recursive function for finding the value for a symbol. Begins looking in the innermost scope,
/// and looks in the outer scopes only if no match is found. If no match is found anywhere, returns
/// None.
fn env_get(s: &str, env: &LinslEnv) -> Option<LinslExpr> {
    match env.inner.get(s) {
        Some(expr) => Some(expr.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(s, outer_env),
                None => None,
            }
        }
    }
}

/// The entry point for evaluating a Linsl program (since every program is an expression).
pub fn evaluate(
    expr: &LinslExpr, 
    pos: PosNum, 
    env: &mut LinslEnv
) -> Result<LinslExpr, LinslErr> {
    match expr {
        LinslExpr::Bool(_) => Ok(expr.clone()),
        LinslExpr::List(exprs) => handle_result(evaluate_list(exprs, env), pos),
        LinslExpr::Number(_) => Ok(expr.clone()),
        LinslExpr::Symbol(s) => 
            env_get(s, env)
            .ok_or(
                LinslErr::SyntaxError(
                    format!("Undefined symbol \'{}\'", s),
                    vec![pos]
                )
            )
        ,
        // None of the other types of expressions are valid as the top level element, which is why
        // they cause an error.
        _ => Err(
            LinslErr::SyntaxError(
                format!("Expected list or atom, found \'{}\'", expr), 
                vec![pos]
            )
        ),
    }
}

fn evaluate_built_in_form(
    expr: &LinslExpr, 
    param_forms: &[LinslExpr], 
    env: &mut LinslEnv
) -> Option<Result<LinslExpr, LinslErr>> {
    match expr {
        LinslExpr::Symbol(s) =>
            match s.as_ref() {
                "define" => Some(evaluate_define(param_forms, env)),
                "if" => Some(evaluate_if(param_forms, env)),
                "lambda" => Some(evaluate_lambda(param_forms)),
                _ => None
            },
            _ => None,
    }
}

/// Evaluation for the primitive "define". It adds a new binding to the inner scope, without
/// evaluating it. In other words, the expression bound to a variable is not evaluated when
/// creating the binding, only when using it.
fn evaluate_define(exprs: &[LinslExpr], env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    if exprs.len() != 2 {
        return Err(
            LinslErr::SyntaxError(
                format!("define must have two forms, found \'{}\'", exprs.len()),
                vec![0]
            )
        );
    };

    let (name_form, val_form) = exprs.split_first()
        .ok_or(LinslErr::InternalError("Could not read define name.".to_string()))?;

    let name: String = match name_form {
        LinslExpr::Symbol(s) => Ok(s.clone()),
        _ => Err(
            LinslErr::SyntaxError(
                format!("First define form must be a symbol, found \'{}\'", name_form),
                vec![1]
                )
            ),
    }?;
    let val = evaluate(&val_form[0], 2, env)?;

    env.inner.insert(name, val);

    Ok(name_form.clone())
}

fn evaluate_forms(forms: &[LinslExpr], env: &mut LinslEnv) -> Result<Vec<LinslExpr>, LinslErr> {
    forms
        .iter()
        .zip(0..)
        .map(|(x, i)| evaluate(x, i, env))
        .collect()
}

/// Evaluation of the primitive "if". It evaluates the first expression passed expecting a boolean b.
/// Then: 
/// - if b it evaluates the first expression after the test expression.
/// - if !b it evaluates the second expression after the test expression.
fn evaluate_if(exprs: &[LinslExpr], env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    if exprs.len() != 3 {
        return Err(
            LinslErr::SyntaxError(
                format!("Expected 3 arguments to if, found {}", exprs.len()),
                vec![0]
            )
        );
    };
    
    let (test_form, body) = exprs.split_first()
        .ok_or(
            LinslErr::InternalError("Could not read if test".to_string())
        )?;
    let test = evaluate(test_form, 1, env)?;
    match test {
        LinslExpr::Bool(b) => {
            if b {
                evaluate(&body[0], 2, env)
            } else {
                evaluate(&body[1], 3, env)
            }
        },
        _ => Err(
            LinslErr::SyntaxError(
                format!("Test form must evaluate to bool, but evaluated to \'{}\'", test),
                vec![1]
            )
        ),
    }
}

/// Evaluation of the primitive "lambda" used to create a closure.
fn evaluate_lambda(expr: &[LinslExpr]) -> Result<LinslExpr, LinslErr> {
    if expr.len() != 2 {
        return Err(
            LinslErr::SyntaxError(
                format!("Lambda must be given two expressions, found {}", expr.len()),
                vec![0]
            )
        );
    };

    let params_form = expr.first().ok_or(
        LinslErr::InternalError("Could not read parameters.".to_string())
    )?;

    let body_form = expr.get(1).ok_or(
        LinslErr::InternalError("Could not read lambda body.".to_string())
    )?;

    Ok(
        LinslExpr::Closure(
            Rc::new(params_form.clone()), 
            Rc::new(body_form.clone()),
        )
    )
}


fn evaluate_list(exprs: &[LinslExpr], env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    let head = exprs
        .first()
        .ok_or(
            LinslErr::SyntaxError(
                "Expected non-empty list".to_string(),
                vec![0]
            ))?;
    let param_forms = &exprs[1..];

    match evaluate_built_in_form(head, param_forms, env) {
        Some(res) => res,
        None => {
            let primitive = evaluate(head, 0, env)?;
            match primitive {
                LinslExpr::Closure(param, body) => {
                    let lambda_env = &mut handle_result(local_env(param, param_forms, env), 1)?;
                    evaluate(&body, 2, lambda_env)
                },
                LinslExpr::Primitive(f) => {
                    let params_eval = param_forms
                        .iter()
                        .zip(1..)
                        .map(|(e, i)| evaluate(e, i, env))
                        .collect::<Result<Vec<LinslExpr>, LinslErr>>();
                    f(&params_eval?)
                },
                _ => Err(
                    LinslErr::SyntaxError(
                        format!("Expected the head of list to be a primitive, found \'{}\'", primitive),
                        vec![0]
                    )
                )
            }
        },
    }
}

fn local_env<'a>(
    new_names: Rc<LinslExpr>,
    vals: &[LinslExpr],
    outer_env: &'a mut LinslEnv
) -> Result<LinslEnv<'a>, LinslErr> {
    let symbs: Vec<String> = parse_list_of_symbols(new_names)?;
    if symbs.len() != vals.len() {
        return Err(
            LinslErr::SyntaxError(
                format!("Expected {} values, found {}", symbs.len(), vals.len()),
                vec![0]
            )
        );
    };

    let vals_eval = evaluate_forms(vals, outer_env)?;
    let mut new_env: HashMap<String, LinslExpr> = HashMap::new();

    for (k, v) in symbs.iter().zip(vals_eval.iter()) {
        new_env.insert(k.clone(), v.clone());
    }

    Ok(
        LinslEnv { 
            inner: new_env, 
            outer: Some(outer_env),
        }
    )
}
