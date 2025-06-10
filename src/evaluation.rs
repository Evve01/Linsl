use crate::{LinslEnv, LinslErr, LinslExpr};

pub fn evaluate(expr: &LinslExpr, env: &mut LinslEnv) -> Result<LinslExpr, LinslErr> {
    match expr {
        LinslExpr::Bool(b) => Ok(LinslExpr::Bool(*b)),
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
    let primitive = evaluate(head, env)?;
    match primitive {
        LinslExpr::Primitive(f) => {
            let params_eval = param_forms
                .iter()
                .map(|e| evaluate(e, env))
                .collect::<Result<Vec<LinslExpr>, LinslErr>>();
            f(&params_eval?)
        }
        _ => Err(
            LinslErr::ListError(
                format!("Expected the head of list to be a primitive, found \'{}\'", primitive)
            )
        )
    }
}
