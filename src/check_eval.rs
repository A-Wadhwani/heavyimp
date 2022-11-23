use std::{collections::HashSet, sync::Mutex};

use crate::{
    evaluator,
    syntax::{Constant, Constant::*, Expr, Statement},
};
use quickcheck::{empty_shrinker, Arbitrary};

lazy_static! {
    static ref NAMES: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

// Quick Checking for the Evaluator

impl Arbitrary for Constant {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Nat(i8::arbitrary(g) as i64)
        } else {
            Bool(bool::arbitrary(g))
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Nat(n) => Box::new(n.shrink().map(Nat)),
            Bool(b) => Box::new(b.shrink().map(Bool)),
        }
    }
}

impl Arbitrary for Expr {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 10 {
            0 => random_reference(g, "").map_or(Self::arbitrary(g), Expr::StoreRead),
            1 => random_reference(g, "h").map_or(Self::arbitrary(g), Expr::HeapRead),
            2..=5 => Expr::Constant(Constant::arbitrary(g)),
            6 => Expr::NatAdd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            7 => Expr::NatLeq(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            8 => Expr::BoolAnd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            9 => Expr::BoolNot(Box::new(Self::arbitrary(g))),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Expr::StoreRead(_) => empty_shrinker(),
            Expr::HeapRead(_) => empty_shrinker(),
            Expr::Constant(_) => Box::new(std::iter::empty()),
            Expr::NatAdd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Expr::NatAdd(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Expr::NatAdd(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Expr::NatLeq(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Expr::NatLeq(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Expr::NatLeq(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Expr::BoolAnd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Expr::BoolAnd(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Expr::BoolAnd(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Expr::BoolNot(e1) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Expr::BoolNot(e1));
                }
                shrinks.push(*e1.clone());
                Box::new(shrinks.into_iter())
            }
        }
    }
}

impl Arbitrary for Statement {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 100 + 1 {
            1..=15 => Statement::StoreAssign(arbitrary_ident(g), Expr::arbitrary(g)),
            16..=30 => Statement::HeapNew(arbitrary_ident(g), Expr::arbitrary(g)),
            31..=35 => random_reference(g, "").map_or(Statement::Skip, |name| {
                Statement::HeapUpdate(name, Expr::arbitrary(g))
            }),
            36..=40 => {
                let alias = arbitrary_ident(g);
                random_reference(g, alias.as_str())
                    .map_or(Statement::Skip, |name| Statement::HeapAlias(alias, name))
            }
            41..=65 => {
                Statement::Sequence(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g)))
            }
            66..=90 => Statement::Conditional(
                Expr::arbitrary(g),
                Box::new(Self::arbitrary(g)),
                Box::new(Self::arbitrary(g)),
            ),
            91..=100 => Statement::While(Expr::arbitrary(g), Box::new(Self::arbitrary(g))),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Statement::StoreAssign(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Statement::StoreAssign(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Statement::HeapNew(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Statement::HeapNew(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Statement::HeapUpdate(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Statement::HeapUpdate(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Statement::HeapAlias(_, _) => empty_shrinker(),
            Statement::Sequence(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Statement::Sequence(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Statement::Sequence(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Statement::Conditional(expr, then_e, else_e) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Statement::Conditional(expr, then_e.clone(), else_e.clone()));
                }
                for then_e in then_e.shrink() {
                    shrinks.push(Statement::Conditional(expr.clone(), then_e, else_e.clone()));
                }
                for else_e in else_e.shrink() {
                    shrinks.push(Statement::Conditional(expr.clone(), then_e.clone(), else_e));
                }
                shrinks.push(*then_e.clone());
                shrinks.push(*else_e.clone());
                Box::new(shrinks.into_iter())
            }
            Statement::While(expr, do_e) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Statement::While(expr, do_e.clone()));
                }
                for do_e in do_e.shrink() {
                    shrinks.push(Statement::While(expr.clone(), do_e));
                }
                shrinks.push(*do_e.clone());
                Box::new(shrinks.into_iter())
            }
            Statement::Skip => empty_shrinker(),
        }
    }
}

// A cleaner to read string
fn arbitrary_ident(g: &mut quickcheck::Gen) -> String {
    let mut s = String::new();
    let mut i = u8::arbitrary(g) % 10;
    while i > 0 {
        // Just letters
        s.push(char::from(b'a' + u8::arbitrary(g) % 26));
        i -= 1;
    }
    NAMES.lock().unwrap().insert(s.clone());
    s
}

fn random_reference(g: &mut quickcheck::Gen, name: &str) -> Option<String> {
    Some(
        g.choose(
            &NAMES
                .lock()
                .unwrap()
                .iter()
                .filter(|n| n != &name)
                .collect::<Vec<_>>(),
        )?
        .to_string(),
    )
}

fn contains_names() -> bool {
    !NAMES.lock().unwrap().is_empty()
}

pub fn quick_check_evaluator(stmnt: Statement) -> bool {
    let val = evaluator::eval_program(&stmnt).is_ok();
    NAMES.lock().unwrap().clear();
    if (val) {
        println!("Passed on {:?}\n", stmnt);
    } else {
        println!("Failed on {:?}\n", stmnt);
    }
    val
}
