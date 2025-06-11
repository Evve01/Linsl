use crate::{LinslEnv, LinslErr, LinslExpr};

pub fn evaluate(expr: &LinslExpr, env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    match expr {
        LinslExpr::Bool(_) => Ok(expr.clone()),
        LinslExpr::Closure(_linsl_expr, _linsl_expr1) => todo!(),
        LinslExpr::List(exprs) => evaluate_list(exprs, env),
        LinslExpr::Number(_) => Ok(expr.clone()),
        LinslExpr::Primitive(_) => Err(LinslErr::SyntaxError(0, 0)),
        LinslExpr::Symbol(k) => 
            env.env.get(k)
            .ok_or(
                LinslErr::PrimitivesError(
                    format!("Undefined symbol \'{}\'", k)
                )
            ).cloned(),
    }
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

fn evaluate_built_in_form(
    expr: &LinslExpr, param_forms: &[LinslExpr], env: &mut LinslEnv
) -> Option<Result<LinslExpr, LinslErr>> {
    match expr {
        LinslExpr::Symbol(s) =>
            match s.as_ref() {
                "if" => Some(evaluate_if(param_forms, env)),
                "define" => Some(evaluate_define(param_forms, env)),
                _ => None
            },
            _ => None,
    }
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

    env.env.insert(name, val);

    Ok(name_form.clone())
}
