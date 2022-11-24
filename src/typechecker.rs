use crate::syntax::{Statement, Expr, Constant};
use crate::error::TypeError;
use std::collections::HashMap;


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Number,
    Location,
    Boolean,
}
fn expect_ty(expected: Type, got: Type) -> Result<Type, TypeError> {
    if expected == got {
        Ok(expected)
    } else {
        Err(TypeError::Mismatch { expected, got })
    }
}

fn typecheck_expr_aux(sigma: &HashMap<String, Type>, ast: &Expr) -> Result<Type, TypeError> {
    match ast {
        Expr::StoreRead(x) => {
            let x_ty = sigma.get(x).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Number, *x_ty)
        }
        Expr::HeapRead(x) => {
            let ix_ty = sigma.get(x).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Location, *ix_ty)
        }
        Expr::Constant(Constant::Nat(_)) => Ok(Type::Number),
        Expr::Constant(Constant::Bool(_)) => Ok(Type::Boolean),
        Expr::NatAdd(a, b) => {
            let a_ty = typecheck_expr_aux(sigma, a)?;
            let b_ty = typecheck_expr_aux(sigma, b)?;
            expect_ty(Type::Number, a_ty)?;
            expect_ty(Type::Number, b_ty)
        }
        Expr::NatLeq(a, b) => {
            let a_ty = typecheck_expr_aux(sigma, a)?;
            let b_ty = typecheck_expr_aux(sigma, b)?;
            expect_ty(Type::Number, a_ty)?;
            expect_ty(Type::Number, b_ty)
        }
        Expr::BoolAnd(a, b) => {
            let a_ty = typecheck_expr_aux(sigma, a)?;
            let b_ty = typecheck_expr_aux(sigma, b)?;
            expect_ty(Type::Boolean, a_ty)?;
            expect_ty(Type::Boolean, b_ty)
        }
        Expr::BoolNot(a) => {
            let a_ty = typecheck_expr_aux(sigma, a)?;
            expect_ty(Type::Boolean, a_ty)
        }
    }
}

fn typecheck_stmt_aux(sigma: &mut HashMap<String, Type>, ast: &Statement) -> Result<(), TypeError> {
    match ast {
        Statement::StoreAssign(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            expect_ty(Type::Number, expr_ty).and_then(|ty| {
                sigma.insert(id.clone(), ty);
                Ok(())
            })
        }
        Statement::HeapNew(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            expect_ty(Type::Location, expr_ty).and_then(|ty| {
                sigma.insert(id.clone(), ty);
                Ok(())
            })
        }
        Statement::HeapUpdate(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            let stored_ty = sigma.get(id).ok_or(TypeError::UnboundVariable)?;
            expect_ty(*stored_ty, expr_ty).map(|_| ())
        }
        Statement::HeapAlias(alias, id) => {
            let stored_ty = sigma.get(id).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Location, *stored_ty).and_then(|ty| {
                sigma.insert(alias.clone(), ty);
                Ok(())
            })
        }
        Statement::Sequence(s1, s2) => {
            typecheck_stmt_aux(sigma, s1)?;
            typecheck_stmt_aux(sigma, s2)
        }
        Statement::Conditional(cond, then, els) => {
            let cond_ty = typecheck_expr_aux(sigma, cond)?;
            expect_ty(Type::Boolean, cond_ty)?;
            typecheck_stmt_aux(sigma, then)?;
            typecheck_stmt_aux(sigma, els)
        }
        Statement::While(cond, luup) => {
            let cond_ty = typecheck_expr_aux(sigma, cond)?;
            expect_ty(Type::Boolean, cond_ty)?;
            typecheck_stmt_aux(sigma, luup)
        }
        Statement::Skip => {
            Ok(())
        }
    }
}
