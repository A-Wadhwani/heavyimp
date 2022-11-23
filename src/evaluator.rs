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
            let value = eval_expr(expr, store, heap)?;
            store.insert(id.clone(), Value::Constant(value));
            Ok(())
        }
        Statement::HeapNew(id, expr) => {
            let value = eval_expr(expr, store, heap)?;
            let index = heap.len();
            heap.push(value);
            store.insert(id.clone(), Value::Location(index));
            Ok(())
        }
        Statement::HeapUpdate(id, expr) => {
            let value = eval_expr(expr, store, heap)?;
            let index = *store.get(id).ok_or(UnboundVariable).and_then(|v| match v {
                Value::Constant(_) => Err(TypeMismatch),
                Value::Location(l) => Ok(l),
            })?;
            // Check if the index is in the heap, and if it is, update it
            heap.get_mut(index)
                .ok_or(InvalidDereference)
                .map(|c| *c = value)
        }
        // Get the location from the store, and add the alias to the store
        Statement::HeapAlias(alias, id) => {
            let index = *store
                .get(id)
                .ok_or(UnboundVariable)
                .and_then(|v| match v {
                    Value::Constant(_) => Err(TypeMismatch),
                    Value::Location(l) => Ok(l),
                })?;
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
            let value = eval_expr(expr, store, heap)?;
            match value {
                Bool(true) => {
                    eval_stmnt(loop_s, store, heap)?;
                    eval_stmnt(stmnt, store, heap)
                }
                Bool(false) => Ok(()),
                _ => Err(TypeMismatch),
            }
        }
        Statement::Skip => Ok(()),
    }
}
