use crate::error::TypeError;
use crate::syntax::{Constant, Expr, Statement};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Number,
    Location,
    Boolean,
}

pub fn typecheck(program: &Statement) -> Result<(), TypeError> {
    let mut sigma = HashMap::new();
    typecheck_stmt_aux(&mut sigma, program)
}

fn expect_ty(expected: Type, got: Type) -> Result<Type, TypeError> {
    if expected == got {
        Ok(expected)
    } else {
        Err(TypeError::Mismatch { expected, got })
    }
}

fn expect_expr_ty(
    expected: Type,
    ast: &Expr,
    sigma: &HashMap<String, Type>,
) -> Result<Type, TypeError> {
    let expr_ty = typecheck_expr_aux(sigma, ast)?;
    expect_ty(expected, expr_ty)
}

fn expect_name_ty(
    expected: Type,
    name: &str,
    sigma: &HashMap<String, Type>,
) -> Result<Type, TypeError> {
    let name_ty = sigma.get(name).unwrap_or(&expected);
    expect_ty(expected, *name_ty)
}

fn typecheck_expr_aux(sigma: &HashMap<String, Type>, ast: &Expr) -> Result<Type, TypeError> {
    match ast {
        Expr::StoreRead(x) => {
            let x_ty = sigma.get(x).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Number, *x_ty)
        }
        Expr::HeapRead(x) => {
            let ix_ty = sigma.get(x).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Location, *ix_ty)?;
            Ok(Type::Number)
        }
        Expr::Constant(Constant::Nat(_)) => Ok(Type::Number),
        Expr::Constant(Constant::Bool(_)) => Ok(Type::Boolean),
        Expr::NatAdd(a, b) => {
            expect_expr_ty(Type::Number, a, sigma)?;
            expect_expr_ty(Type::Number, b, sigma)
        }
        Expr::NatLeq(a, b) => {
            expect_expr_ty(Type::Number, a, sigma)?;
            expect_expr_ty(Type::Number, b, sigma)?;
            Ok(Type::Boolean)
        }
        Expr::BoolAnd(a, b) => {
            expect_expr_ty(Type::Boolean, a, sigma)?;
            expect_expr_ty(Type::Boolean, b, sigma)?;
            Ok(Type::Boolean)
        }
        Expr::BoolNot(a) => {
            expect_expr_ty(Type::Boolean, a, sigma)?;
            Ok(Type::Boolean)
        }
    }
}

fn typecheck_stmt_aux(sigma: &mut HashMap<String, Type>, ast: &Statement) -> Result<(), TypeError> {
    match ast {
        Statement::StoreAssign(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            expect_name_ty(Type::Number, id, sigma)?;
            expect_ty(Type::Number, expr_ty).and_then(|_| {
                sigma.insert(id.clone(), Type::Number);
                Ok(())
            })
        }
        Statement::HeapNew(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            expect_name_ty(Type::Location, id, sigma)?;
            expect_ty(Type::Number, expr_ty).and_then(|_| {
                sigma.insert(id.clone(), Type::Location);
                Ok(())
            })
        }
        Statement::HeapUpdate(id, expr) => {
            let expr_ty = typecheck_expr_aux(sigma, expr)?;
            let id_ty = sigma.get(id).ok_or(TypeError::UnboundVariable)?;
            expect_ty(Type::Location, *id_ty).map(|_| ())?;
            expect_ty(Type::Number, expr_ty).map(|_| ())
        }
        Statement::HeapAlias(alias, id) => {
            let stored_ty = sigma.get(id).ok_or(TypeError::UnboundVariable)?;
            expect_name_ty(Type::Location, alias, sigma)?;
            expect_ty(Type::Location, *stored_ty).and_then(|_| {
                sigma.insert(alias.clone(), Type::Location);
                Ok(())
            })
        }
        Statement::Sequence(s1, s2) => {
            typecheck_stmt_aux(sigma, s1)?;
            typecheck_stmt_aux(sigma, s2)
        }
        Statement::Conditional(cond, then, els) => {
            expect_expr_ty(Type::Boolean, cond, sigma)?;
            // The variables bound in the then and else branches are disjoint and don't leak out.
            let mut then_sigma = sigma.clone();
            let mut els_sigma = sigma.clone();
            typecheck_stmt_aux(&mut then_sigma, then)?;
            typecheck_stmt_aux(&mut els_sigma, els)?;
            *sigma = then_sigma
                .into_iter()
                .filter_map(|(k, v1)| {
                    if let Some(v2) = els_sigma.get(&k) {
                        if &v1 == v2 {
                            Some((k, *v2))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();
            Ok(())
        }
        Statement::While(cond, luup) => {
            expect_expr_ty(Type::Boolean, cond, sigma)?;
            let mut luup_sigma = sigma.clone();
            typecheck_stmt_aux(&mut luup_sigma, luup)
        }
        Statement::Skip => Ok(()),
    }
}

#[allow(unused)]
mod test {
    use super::*;
    use crate::syntax::{Constant::*, *};

    #[test]
    fn basic_test() -> Result<(), TypeError> {
        let program = Statement::Sequence(
            Box::new(Statement::HeapNew("x".into(), Expr::Constant(Nat(1)))),
            Box::new(Statement::Sequence(
                Box::new(Statement::Sequence(
                    Box::new(Statement::HeapNew("z".into(), Expr::Constant(Nat(2)))),
                    Box::new(Statement::HeapUpdate(
                        "z".into(),
                        Expr::NatAdd(
                            Box::new(Expr::HeapRead("x".into())),
                            Box::new(Expr::HeapRead("z".into())),
                        ),
                    )),
                )),
                Box::new(Statement::Conditional(
                    Expr::NatLeq(
                        Box::new(Expr::HeapRead("x".into())),
                        Box::new(Expr::Constant(Nat(0))),
                    ),
                    Box::new(Statement::HeapNew("y".into(), Expr::HeapRead("z".into()))),
                    Box::new(Statement::HeapNew("y".into(), Expr::Constant(Nat(4)))),
                )),
            )),
        );
        typecheck(&program)
    }

    #[test]
    fn basic_test2() {
        let program = Statement::Sequence(
            Box::new(Statement::Sequence(
                Box::new(Statement::Sequence(
                    Box::new(Statement::Sequence(
                        Box::new(Statement::Sequence(
                            Box::new(Statement::Sequence(
                                Box::new(Statement::Skip),
                                Box::new(Statement::StoreAssign(
                                    "x1".into(),
                                    Expr::Constant(Nat(13)),
                                )),
                            )),
                            Box::new(Statement::Skip),
                        )),
                        Box::new(Statement::HeapNew(
                            "x2".into(),
                            Expr::StoreRead("x1".into()),
                        )),
                    )),
                    Box::new(Statement::Conditional(
                        Expr::BoolNot(Box::new(Expr::Constant(Bool(true)))),
                        Box::new(Statement::While(
                            Expr::BoolAnd(
                                Box::new(Expr::BoolAnd(
                                    Box::new(Expr::Constant(Bool(false))),
                                    Box::new(Expr::Constant(Bool(true))),
                                )),
                                Box::new(Expr::Constant(Bool(true))),
                            ),
                            Box::new(Statement::StoreAssign(
                                "x4".into(),
                                Expr::StoreRead("x1".into()),
                            )),
                        )),
                        Box::new(Statement::HeapAlias("x1".into(), "x2".into())),
                    )),
                )),
                Box::new(Statement::HeapNew("x3".into(), Expr::Constant(Nat(117)))),
            )),
            Box::new(Statement::HeapUpdate(
                "x2".into(),
                Expr::StoreRead("x1".into()),
            )),
        );
        typecheck(&program).unwrap_err();
    }
}
