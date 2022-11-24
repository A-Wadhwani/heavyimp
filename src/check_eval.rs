use std::{collections::HashSet, sync::Mutex};

use crate::{
    error::EvalError,
    evaluator,
    syntax::{Constant, Constant::*, Expr, Statement},
};
use quickcheck::{empty_shrinker, Arbitrary, TestResult};

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
            0 => random_reference(g, "").map_or(Self::arbitrary(g), Self::StoreRead),
            1 => random_reference(g, "").map_or(Self::arbitrary(g), Self::HeapRead),
            2..=5 => Self::Constant(Constant::arbitrary(g)),
            6 => Self::NatAdd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            7 => Self::NatLeq(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            8 => Self::BoolAnd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            9 => Self::BoolNot(Box::new(Self::arbitrary(g))),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::StoreRead(_) => empty_shrinker(),
            Self::HeapRead(_) => empty_shrinker(),
            Self::Constant(_) => Box::new(std::iter::empty()),
            Self::NatAdd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::NatAdd(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::NatAdd(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::NatLeq(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::NatLeq(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::NatLeq(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::BoolAnd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::BoolAnd(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::BoolAnd(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::BoolNot(e1) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::BoolNot(e1));
                }
                shrinks.push(*e1.clone());
                Box::new(shrinks.into_iter())
            }
        }
    }
}

impl Arbitrary for Statement {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut stmnt = Self::generate_stmnts(g);
        // Ensure we have a statment of big-enough size
        while stmnt.size() < g.size() {
            stmnt = Self::Sequence(Box::new(stmnt), Box::new(Self::generate_stmnts(g)));
        }
        stmnt
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::StoreAssign(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Self::StoreAssign(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Self::HeapNew(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Self::HeapNew(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Self::HeapUpdate(id, expr) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Self::HeapUpdate(id.clone(), expr));
                }
                Box::new(shrinks.into_iter())
            }
            Self::HeapAlias(_, _) => empty_shrinker(),
            Self::Sequence(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::Sequence(e1, e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::Sequence(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::Conditional(expr, then_e, else_e) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Self::Conditional(expr, then_e.clone(), else_e.clone()));
                }
                for then_e in then_e.shrink() {
                    shrinks.push(Self::Conditional(expr.clone(), then_e, else_e.clone()));
                }
                for else_e in else_e.shrink() {
                    shrinks.push(Self::Conditional(expr.clone(), then_e.clone(), else_e));
                }
                shrinks.push(*then_e.clone());
                shrinks.push(*else_e.clone());
                Box::new(shrinks.into_iter())
            }
            Self::While(expr, do_e) => {
                let mut shrinks = Vec::new();
                for expr in expr.shrink() {
                    shrinks.push(Self::While(expr, do_e.clone()));
                }
                for do_e in do_e.shrink() {
                    shrinks.push(Self::While(expr.clone(), do_e));
                }
                shrinks.push(*do_e.clone());
                Box::new(shrinks.into_iter())
            }
            Self::Skip => empty_shrinker(),
        }
    }
}

impl Statement {
    fn generate_stmnts(g: &mut quickcheck::Gen) -> Statement {
        match u8::arbitrary(g) % 105 + 1 {
            1..=15 => Self::StoreAssign(arbitrary_ident(g), Expr::arbitrary(g)),
            16..=30 => Self::HeapNew(arbitrary_ident(g), Expr::arbitrary(g)),
            31..=35 => match random_reference(g, "") {
                Some(r) => Self::HeapUpdate(r, Expr::arbitrary(g)),
                None => Self::generate_stmnts(g),
            },
            36..=45 => {
                let alias = arbitrary_ident(g);
                match random_reference(g, "") {
                    Some(r) => Self::HeapAlias(alias, r),
                    None => Self::generate_stmnts(g),
                }
            }
            46..=65 => Self::Sequence(
                Box::new(Self::generate_stmnts(g)),
                Box::new(Self::generate_stmnts(g)),
            ),
            66..=90 => Self::Conditional(
                Expr::arbitrary(g),
                Box::new(Self::generate_stmnts(g)),
                Box::new(Self::generate_stmnts(g)),
            ),
            91..=100 => Self::While(Expr::arbitrary(g), Box::new(Self::generate_stmnts(g))),
            101..=105 => Self::Skip,
            _ => unreachable!(),
        }
    }

    // Size of a statement is the number of statements in the sequence
    fn size(&self) -> usize {
        match self {
            Self::StoreAssign(_, _) => 1,
            Self::HeapNew(_, _) => 1,
            Self::HeapUpdate(_, _) => 1,
            Self::HeapAlias(_, _) => 1,
            Self::Sequence(e1, e2) => e1.size() + e2.size(),
            Self::Conditional(_, then_e, else_e) => then_e.size() + else_e.size(),
            Self::While(_, do_e) => do_e.size(),
            Self::Skip => 1,
        }
    }
}

// A cleaner to read string
fn arbitrary_ident(g: &mut quickcheck::Gen) -> String {
    let mut s = String::new();
    // Occasionally use a random reference

    let mut i = u8::arbitrary(g) % 10 + 4;
    while i > 0 {
        // Just letters
        s.push(char::from(b'a' + u8::arbitrary(g) % 26));
        i -= 1;
    }
    NAMES.lock().unwrap().insert(s.clone());
    if bool::arbitrary(g) {
        random_reference(g, "").unwrap_or(s)
    } else {
        s
    }
}

fn random_reference(g: &mut quickcheck::Gen, name: &str) -> Option<String> {
    Some(
        (*g.choose(
            &NAMES
                .lock()
                .unwrap()
                .iter()
                .filter(|n| n != &name)
                .collect::<Vec<_>>(),
        )?)
        .to_string(),
    )
}

pub fn quick_check_evaluator(stmnt: Statement) -> TestResult {
    // Check if it passes the type-checker here
    if false {
        println!("Discarding Test: {:?}\n", stmnt);
        // This test can be ignored if it does
        return TestResult::discard();
    }
    NAMES.lock().unwrap().clear();
    let val = evaluator::eval_program(&stmnt);
    if val.is_ok() {
        println!("Passed on {:?}\n", stmnt);
        TestResult::passed()
    } else {
        let err = val.unwrap_err();
        println!("{:?} error on {:?}\n", err, stmnt);
        TestResult::failed()
    }
}
