use std::collections::HashMap;

use crate::error::{EvalError::*, EvalResult};
use crate::syntax::{Constant::*, *};

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Number(i64),
    Location(usize),
}

type Sigma = HashMap<Ident, Value>;
type Heap = Vec<i64>;

pub fn eval_program(program: &Statement) -> EvalResult<(Sigma, Heap)> {
    let mut store = HashMap::new();
    let mut heap = Vec::new();
    eval_stmnt(program, &mut store, &mut heap)?;
    Ok((store, heap))
}

fn eval_expr(expr: &Expr, store: &Sigma, heap: &Heap) -> EvalResult<Constant> {
    match expr {
        // Read from the store, and return if it's a constant
        Expr::StoreRead(x) => store.get(x).ok_or(UnboundVariable).and_then(|v| match v {
            Value::Number(i) => Ok(Nat(*i)),
            Value::Location(_) => Err(TypeMismatch),
        }),
        // Get the location from the store, and read from the heap
        Expr::HeapRead(x) => {
            let index = store.get(x).ok_or(UnboundVariable).and_then(get_loc)?;
            let value = heap.get(index).ok_or(InvalidDereference)?;
            Ok(Constant::Nat(*value))
        }
        // Return the constant
        Expr::Constant(c) => Ok(*c),
        // Evaluate expressions if they're the correct values
        Expr::NatAdd(a, b) => {
            let a = eval_expr(a, store, heap)?;
            let b = eval_expr(b, store, heap)?;
            match (a, b) {
                (Nat(a), Nat(b)) => Ok(Nat(a + b)),
                _ => Err(TypeMismatch),
            }
        }
        Expr::NatLeq(a, b) => {
            let a = eval_expr(a, store, heap)?;
            let b = eval_expr(b, store, heap)?;
            match (a, b) {
                (Nat(a), Nat(b)) => Ok(Bool(a <= b)),
                _ => Err(TypeMismatch),
            }
        }
        Expr::BoolAnd(a, b) => {
            let a = eval_expr(a, store, heap)?;
            let b = eval_expr(b, store, heap)?;
            match (a, b) {
                (Bool(a), Bool(b)) => Ok(Bool(a && b)),
                _ => Err(TypeMismatch),
            }
        }
        Expr::BoolNot(a) => {
            let a = eval_expr(a, store, heap)?;
            match a {
                Bool(a) => Ok(Bool(!a)),
                _ => Err(TypeMismatch),
            }
        }
    }
}

fn eval_stmnt(stmnt: &Statement, store: &mut Sigma, heap: &mut Heap) -> EvalResult<()> {
    match stmnt {
        Statement::StoreAssign(id, expr) => {
            let value = eval_expr(expr, store, heap).and_then(get_nat)?;
            // If value is present, make sure it's a number
            match store.get(id) {
                Some(Value::Number(_)) | None => store.insert(id.clone(), Value::Number(value)),
                Some(Value::Location(_)) => Err(BoundTypeMismatch)?,
            };
            Ok(())
        }
        Statement::HeapNew(id, expr) => {
            let value = eval_expr(expr, store, heap).and_then(get_nat)?;
            let index = heap.len();
            heap.push(value);
            // If value is present, make sure it's a location
            match store.get(id) {
                Some(Value::Location(_)) | None => store.insert(id.clone(), Value::Location(index)),
                Some(Value::Number(_)) => Err(BoundTypeMismatch)?,
            };
            Ok(())
        }
        Statement::HeapUpdate(id, expr) => {
            let value = eval_expr(expr, store, heap).and_then(get_nat)?;
            let index = store.get(id).ok_or(UnboundVariable).and_then(get_loc)?;
            // Check if the index is in the heap, and if it is, update it
            heap.get_mut(index)
                .ok_or(InvalidDereference)
                .map(|c| *c = value)
        }
        // Get the location from the store, and add the alias to the store
        Statement::HeapAlias(alias, id) => {
            let index = store.get(id).ok_or(UnboundVariable).and_then(get_loc)?;
            store.insert(alias.clone(), Value::Location(index));
            Ok(())
        }
        Statement::Sequence(s1, s2) => {
            eval_stmnt(s1, store, heap)?;
            eval_stmnt(s2, store, heap)
        }
        Statement::Conditional(expr, then_s, else_s) => {
            let value = eval_expr(expr, store, heap)?;
            match value {
                Bool(true) => eval_stmnt(then_s, store, heap),
                Bool(false) => eval_stmnt(else_s, store, heap),
                _ => Err(TypeMismatch),
            }
        }
        Statement::While(expr, loop_s) => {
            let mut value = eval_expr(expr, store, heap)?;
            let mut count = 0;
            while let Bool(true) = value {
                if count > 1000 {
                    // We don't want to loop forever, automatically break here
                    return Ok(());
                }
                eval_stmnt(loop_s, store, heap)?;
                value = eval_expr(expr, store, heap)?;
                count += 1;
            }
            if matches!(value, Bool(_)) {
                Ok(())
            } else {
                Err(TypeMismatch)
            }
        }
        Statement::Skip => Ok(()),
    }
}

const fn get_nat(c: Constant) -> EvalResult<i64> {
    match c {
        Nat(i) => Ok(i),
        Bool(_) => Err(TypeMismatch),
    }
}

const fn get_loc(v: &Value) -> EvalResult<usize> {
    match v {
        Value::Number(_) => Err(TypeMismatch),
        Value::Location(l) => Ok(*l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiple_stores() {
        let program = Statement::Sequence(
            Box::new(Statement::StoreAssign("x".into(), Expr::Constant(Nat(1)))),
            Box::new(Statement::StoreAssign("y".into(), Expr::Constant(Nat(2)))),
        );
        let (store, heap) = eval_program(&program).unwrap();
        assert_eq!(store.get("x"), Some(&Value::Number(1)));
        assert_eq!(store.get("y"), Some(&Value::Number(2)));
        assert_eq!(heap.len(), 0);
    }

    #[test]
    fn test_heap_and_store() {
        let program = Statement::Sequence(
            Box::new(Statement::StoreAssign("x".into(), Expr::Constant(Nat(1)))),
            Box::new(Statement::HeapNew("y".into(), Expr::Constant(Nat(2)))),
        );
        let (store, heap) = eval_program(&program).unwrap();
        assert_eq!(store.get("x"), Some(&Value::Number(1)));
        assert_eq!(store.get("y"), Some(&Value::Location(0)));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap[0], 2);
    }

    #[test]
    fn test_heap_assigns() {
        let program = Statement::Sequence(
            Box::new(Statement::StoreAssign("x".into(), Expr::Constant(Nat(1)))),
            Box::new(Statement::Sequence(
                Box::new(Statement::HeapNew("y".into(), Expr::StoreRead("x".into()))),
                Box::new(Statement::Sequence(
                    Box::new(Statement::HeapAlias("z".into(), "y".into())),
                    Box::new(Statement::HeapUpdate("z".into(), Expr::Constant(Nat(3)))),
                )),
            )),
        );
        let (store, heap) = eval_program(&program).unwrap();
        assert_eq!(store.get("x"), Some(&Value::Number(1)));
        assert_eq!(store.get("y"), Some(&Value::Location(0)));
        assert_eq!(store.get("z"), Some(&Value::Location(0)));
        assert_eq!(heap.len(), 1);
        assert_eq!(heap[0], 3);
    }

    #[test]
    fn test_heap_dereference() {
        let program = Statement::Sequence(
            Box::new(Statement::StoreAssign("x".into(), Expr::Constant(Nat(1)))),
            Box::new(Statement::Sequence(
                Box::new(Statement::Sequence(
                    Box::new(Statement::HeapNew("z".into(), Expr::Constant(Nat(2)))),
                    Box::new(Statement::HeapUpdate(
                        "z".into(),
                        Expr::NatAdd(
                            Box::new(Expr::StoreRead("x".into())),
                            Box::new(Expr::HeapRead("z".into())),
                        ),
                    )),
                )),
                Box::new(Statement::HeapNew("y".into(), Expr::HeapRead("z".into()))),
            )),
        );
        let (store, heap) = eval_program(&program).unwrap();
        assert_eq!(store.get("x"), Some(&Value::Number(1)));
        assert_eq!(store.get("y"), Some(&Value::Location(1)));
        assert_eq!(store.get("z"), Some(&Value::Location(0)));
        assert_eq!(heap.len(), 2);
        assert_eq!(heap[0], 3);
        assert_eq!(heap[1], 3);
    }

    #[test]
    fn test_conditional_heap() {
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
        let (store, heap) = eval_program(&program).unwrap();
        assert_eq!(store.get("x"), Some(&Value::Location(0)));
        assert_eq!(store.get("y"), Some(&Value::Location(2)));
        assert_eq!(store.get("z"), Some(&Value::Location(1)));
        assert_eq!(heap.len(), 3);
        assert_eq!(heap[0], 1);
        assert_eq!(heap[1], 3);
        assert_eq!(heap[2], 4);
    }
}
