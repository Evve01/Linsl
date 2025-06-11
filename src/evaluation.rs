use std::{collections::HashMap, rc::Rc};

use crate::{parsing::parse_list_of_symbols, LinslEnv, LinslErr, LinslExpr};

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

pub fn evaluate(expr: &LinslExpr, env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    match expr {
        LinslExpr::Bool(_) => Ok(expr.clone()),
        LinslExpr::Closure(_, _) => Err(LinslErr::SyntaxError(0, 0)),
        LinslExpr::List(exprs) => evaluate_list(exprs, env),
        LinslExpr::Number(_) => Ok(expr.clone()),
        LinslExpr::Primitive(_) => Err(LinslErr::SyntaxError(0, 0)),
        LinslExpr::Symbol(s) => 
            env_get(s, env)
            .ok_or(
                LinslErr::PrimitivesError(
                    format!("Undefined symbol \'{}\'", s)
                )
            )
        ,
    }
}

fn evaluate_built_in_form(
    expr: &LinslExpr, param_forms: &[LinslExpr], env: &mut LinslEnv
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

fn evaluate_define(exprs: &[LinslExpr], env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    if exprs.len() != 2 {
        return Err(
            LinslErr::PrimitivesError(
                format!("define must have two forms, found \'{}\'", exprs.len())
            )
        );
    };

    let (name_form, val_form) = exprs.split_first()
        .ok_or(LinslErr::InternalError("Could not read define name.".to_string()))?;

    let name: String = match name_form {
        LinslExpr::Symbol(s) => Ok(s.clone()),
        _ => Err(
            LinslErr::PrimitivesError(
                format!("First define form must be a symbol, found \'{}\'", name_form)
                )
            ),
    }?;
    let val = evaluate(&val_form[0], env)?;

    env.inner.insert(name, val);

    Ok(name_form.clone())
}

fn evaluate_forms(forms: &[LinslExpr], env: &mut LinslEnv) -> Result<Vec<LinslExpr>, LinslErr> {
    forms
        .iter()
        .map(|x| evaluate(x, env))
        .collect()
}

fn evaluate_if(exprs: &[LinslExpr], env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    if exprs.len() != 3 {
        return Err(LinslErr::PrimitivesError(format!("Expected 3 arguments to if, found {}", exprs.len())));
    };
    
    let (test_form, body) = exprs.split_first()
        .ok_or(
            LinslErr::InternalError("Could not read if test".to_string())
        )?;
    let test = evaluate(test_form, env)?;
    match test {
        LinslExpr::Bool(b) => {
            if b {
                evaluate(&body[0], env)
            } else {
                evaluate(&body[1], env)
            }
        },
        _ => Err(
            LinslErr::PrimitivesError(
                format!("Test form must evaluate to bool, but evaluated to \'{}\'", test)
            )
        ),
    }
}

fn evaluate_lambda(expr: &[LinslExpr]) -> Result<LinslExpr, LinslErr> {
    if expr.len() != 2 {
        return Err(
            LinslErr::PrimitivesError(
                format!("Lambda must be given two expressions, found {}", expr.len())
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
        .ok_or(LinslErr::PrimitivesError("Expected non-empty list".to_string()))?;
    let param_forms = &exprs[1..];

    match evaluate_built_in_form(head, param_forms, env) {
        Some(res) => res,
        None => {
            let primitive = evaluate(head, env)?;
            match primitive {
                LinslExpr::Closure(param, body) => {
                    let lambda_env = &mut local_env(param, param_forms, env)?;
                    evaluate(&body, lambda_env)
                },
                LinslExpr::Primitive(f) => {
                    let params_eval = param_forms
                        .iter()
                        .map(|e| evaluate(e, env))
                        .collect::<Result<Vec<LinslExpr>, LinslErr>>();
                    f(&params_eval?)
                },
                _ => Err(
                    LinslErr::ListError(
                        format!("Expected the head of list to be a primitive, found \'{}\'", primitive)
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
            LinslErr::PrimitivesError(
                format!("Expected {} values, found {}", symbs.len(), vals.len())
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
