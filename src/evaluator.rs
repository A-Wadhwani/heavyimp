use std::collections::HashMap;

use crate::error::{EvalError::*, EvalResult};
use crate::syntax::{Constant::*, *};

enum Value {
    Constant(Constant),
    Location(usize),
}

type Sigma = HashMap<Ident, Value>;
type Heap = Vec<Constant>;

fn eval_expr(expr: &Expr, store: &Sigma, heap: &Heap) -> EvalResult<Constant> {
    match expr {
        // Read from the store, and return if it's a constant
        Expr::StoreRead(x) => store.get(x).ok_or(UnboundVariable).and_then(|v| match v {
            Value::Constant(c) => Ok(c.clone()),
            Value::Location(_) => Err(TypeMismatch),
        }),
        // Get the location from the store, and read from the heap
        Expr::HeapRead(x) => {
            let index = *store.get(x).ok_or(UnboundVariable).and_then(|v| match v {
                Value::Constant(_) => Err(TypeMismatch),
                Value::Location(l) => Ok(l),
            })?;
            heap.get(index).ok_or(InvalidDereference).cloned()
        }
        // Return the constant
        Expr::Constant(c) => Ok(c.clone()),
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
