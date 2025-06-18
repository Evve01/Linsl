//! Code for evaluating Linsl expressions.

use crate::datatypes::{LinslEnv, LinslErr, LinslExpr, LinslRes, PosNum};
use crate::parsing::{handle_result, parse_list_of_symbols};

/// Creates new bindings within the environment specified. For example, given the list of symbols
/// (a b c) and the list of values (1 2 3) it will bind a to 1, b to 2 and c to 3.
///
/// If the list of values is longer than the list of symbols, the last symbol will be bound to the
/// list of remaining values. For example, given the list of symbols (a b c) and the list of values
/// (1 2 3 4), it will bind a to 1, b to 2 and c to (3 4).
///
/// If the list of values is shorter than the list of symbols, will generate an error.
fn bind<'a>(
    symbs: &LinslExpr,
    vals: &LinslExpr,
    env: &'a mut LinslEnv
) -> Result<LinslEnv<'a>, LinslErr> {
    let symbs_vec: Vec<String> = parse_list_of_symbols(symbs)?;
    let vals_vec: Vec<LinslExpr> = match vals {
        LinslExpr::List(v) => Ok(v.clone()),
        _ => Err(
            LinslErr::SyntaxError(
                "Expected list of values.".to_string(),
                vec![2]
            )
        ),
    }?;

    if symbs_vec.is_empty() {
        return Ok(env.clone());
    };

    if symbs_vec.len() > vals_vec.len() {
        return Err(
            LinslErr::SyntaxError(
                format!("Got {} symbols and {} values; cannot have more symbols than values.",
                    symbs_vec.len(),
                    vals_vec.len()),
                vec![1]
            )
        );
    };

    for (k, v) in symbs_vec.iter().zip(vals_vec.iter()) {
            env.inner.insert(k.clone(), v.clone());
    };
    if symbs_vec.len() < vals_vec.len() {
        let i = symbs_vec.len();
        let (_, vals_rest) = vals_vec.split_at(i - 1);
        env.inner.insert(symbs_vec[i-1].clone(), LinslExpr::List(vals_rest.to_vec()));
    };
    Ok(env.clone())
}

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
) -> LinslRes {
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
) -> Option<LinslRes> {
    match expr {
        LinslExpr::Symbol(s) =>
            match s.as_ref() {
                "define" => Some(evaluate_define(param_forms, env)),
                "if" => Some(evaluate_if(param_forms, env)),
                "lambda" => Some(evaluate_lambda(param_forms)),
                "macro" => Some(evaluate_macro(param_forms)),
                "quote" => match param_forms.first() {
                    Some(e) => Some(Ok(e.clone())),
                    None => Some(
                        Err(LinslErr::SyntaxError(
                            "Found no expression to quote.".to_string(), 
                            vec![0])
                        )
                    ),
                }
                _ => None
            },
            _ => None,
    }
}

/// Evaluation for the primitive "define". It adds a new binding to the inner scope, by
/// evaluating the second expression, and associating the first (which mus tbe a symbol) with the
/// returned value.
fn evaluate_define(exprs: &[LinslExpr], env: &mut LinslEnv) -> LinslRes {
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

/// Evaluation of the primitive "if". It evaluates the first expression passed expecting a boolean
/// b.
/// Then: 
/// - if b it evaluates the first expression after the test expression.
/// - if !b it evaluates the second expression after the test expression.
fn evaluate_if(exprs: &[LinslExpr], env: &mut LinslEnv) -> LinslRes {
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
fn evaluate_lambda(expr: &[LinslExpr]) -> LinslRes {
    let (params_form, body_form) = get_params_and_body(expr)?;
    Ok(
        LinslExpr::Closure(
            Box::new(params_form),
            Box::new(body_form),
        )
    )
}


fn evaluate_list(exprs: &[LinslExpr], env: &mut LinslEnv) -> LinslRes {
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
                    let evals = LinslExpr::List(evaluate_forms(
                                param_forms,
                                env)?);
                    let mut new_env = LinslEnv::new(env);
                    let lambda_env = &mut handle_result(
                        bind(
                            &param, 
                            &evals,
                            &mut new_env
                            ),
                        1
                        )?;
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
                LinslExpr::Macro(param, body) => {
                    let e = env.clone();
                    let mut new_env = LinslEnv::new(&e);
                    let macro_env = &mut handle_result(
                        bind(
                            &param,
                            &LinslExpr::List(param_forms.to_vec()),
                            &mut new_env
                        ),
                    1)?;
                    evaluate(&evaluate(&body, 2, macro_env)?, 1, env)
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

fn evaluate_macro(exprs: &[LinslExpr]) -> LinslRes {
    let (params_form, body_form) = get_params_and_body(exprs)?;
    Ok(
        LinslExpr::Macro(
            Box::new(params_form),
            Box::new(body_form)
        )
    )
}

fn get_params_and_body(exprs: &[LinslExpr]) -> Result<(LinslExpr, LinslExpr), LinslErr> {
    if exprs.len() != 2 {
        return Err(
            LinslErr::SyntaxError(
                format!("Lambda must be given two expressions, found {}", exprs.len()),
                vec![0]
            )
        );
    };

    let params_form = exprs.first().ok_or(
        LinslErr::InternalError("Could not read parameters.".to_string())
    )?;

    let body_form = exprs.get(1).ok_or(
        LinslErr::InternalError("Could not read lambda body.".to_string())
    )?;

    Ok((params_form.clone(), body_form.clone()))
}
