use std::{any::type_name, collections::HashSet, sync::Mutex};

use crate::{
    evaluator::{self, eval_program},
    syntax::{Constant, Constant::*, Expr, Statement},
    typechecker::typecheck,
};
use quickcheck::{empty_shrinker, Arbitrary, Gen, TestResult};

lazy_static! {
    static ref STORE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref HEAP: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

// Quick Checking for the Evaluator

impl Constant {
    fn arbitrary_int(g: &mut Gen) -> Self {
        Nat(i8::arbitrary(g) as i64)
    }

    fn arbitrary_bool(g: &mut Gen) -> Self {
        Bool(bool::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Nat(n) => Box::new(n.shrink().map(Nat)),
            Bool(b) => Box::new(b.shrink().map(Bool)),
        }
    }
}

impl Expr {
    // Generate a nat expression
    fn arbitrary_nat(g: &mut Gen) -> Self {
        let constant = Self::Constant(Constant::arbitrary_int(g));
        match u8::arbitrary(g) % 4 {
            0 => random_store(g).map_or(constant, Self::StoreRead),
            1 => random_heap(g).map_or(constant, Self::HeapRead),
            2 => constant,
            3 => Self::NatAdd(
                Box::new(Self::arbitrary_nat(g)),
                Box::new(Self::arbitrary_nat(g)),
            ),
            _ => unreachable!(),
        }
    }

    fn arbitrary_bool(g: &mut Gen) -> Self {
        match u8::arbitrary(g) % 4 {
            0 => Self::NatLeq(
                Box::new(Self::arbitrary_nat(g)),
                Box::new(Self::arbitrary_nat(g)),
            ),
            1 => Self::BoolAnd(
                Box::new(Self::arbitrary_bool(g)),
                Box::new(Self::arbitrary_bool(g)),
            ),
            2 => Self::BoolNot(Box::new(Self::arbitrary_bool(g))),
            3 => Self::Constant(Constant::arbitrary_bool(g)),
            _ => unreachable!(),
        }
    }

    fn arbitrary_store(g: &mut Gen, s: &String) -> Self {
        STORE.lock().unwrap().remove(s);
        let res = Self::arbitrary_nat(g);
        STORE.lock().unwrap().insert(s.clone());
        res
    }

    fn arbitrary_heap(g: &mut Gen, s: &String) -> Self {
        HEAP.lock().unwrap().remove(s);
        let res = Self::arbitrary_nat(g);
        HEAP.lock().unwrap().insert(s.clone());
        res
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        match self {
            Self::StoreRead(_) => empty_shrinker(),
            Self::HeapRead(_) => empty_shrinker(),
            Self::Constant(c) => Box::new(c.shrink().map(Self::Constant)),
            Self::NatAdd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::NatAdd(Box::new(e1), e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::NatAdd(e1.clone(), Box::new(e2)));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::NatLeq(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::NatLeq(Box::new(e1), e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::NatLeq(e1.clone(), Box::new(e2)));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::BoolAnd(e1, e2) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::BoolAnd(Box::new(e1), e2.clone()));
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::BoolAnd(e1.clone(), Box::new(e2)));
                }
                shrinks.push(*e1.clone());
                shrinks.push(*e2.clone());
                Box::new(shrinks.into_iter())
            }
            Self::BoolNot(e1) => {
                let mut shrinks = Vec::new();
                for e1 in e1.shrink() {
                    shrinks.push(Self::BoolNot(Box::new(e1)));
                }
                shrinks.push(*e1.clone());
                Box::new(shrinks.into_iter())
            }
        }
    }
}

impl Arbitrary for Statement {
    fn arbitrary(g: &mut Gen) -> Self {
        STORE.lock().unwrap().clear();
        HEAP.lock().unwrap().clear();
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
                    shrinks.push(*e1);
                }
                for e2 in e2.shrink() {
                    shrinks.push(Self::Sequence(e1.clone(), e2));
                }
                shrinks.push(*e1.clone());
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
    fn generate_stmnts(g: &mut Gen) -> Statement {
        match u8::arbitrary(g) % 105 + 1 {
            1..=15 => {
                let id = arbitrary_ident(g, true);
                let expr = Expr::arbitrary_store(g, &id);
                Self::StoreAssign(id, expr)
            }
            16..=30 => {
                let id = arbitrary_ident(g, false);
                let expr = Expr::arbitrary_heap(g, &id);
                Self::HeapNew(id, expr)
            }
            31..=35 => match random_heap(g) {
                Some(r) => {
                    let expr = Expr::arbitrary_heap(g, &r);
                    Self::HeapUpdate(r, expr)
                }
                None => Self::generate_stmnts(g),
            },
            36..=45 => match random_heap(g) {
                Some(r) => {
                    let alias = arbitrary_ident(g, false);
                    Self::HeapAlias(alias, r)
                }
                None => Self::generate_stmnts(g),
            },
            46..=65 => Self::Sequence(
                Box::new(Self::generate_stmnts(g)),
                Box::new(Self::generate_stmnts(g)),
            ),
            66..=90 => {
                let sets = clone_sets();
                let cond = Expr::arbitrary_bool(g);
                let then_e = Self::generate_stmnts(g);
                restore_sets(&sets);
                let else_e = Self::generate_stmnts(g);
                restore_sets(&sets);
                Self::Conditional(cond, Box::new(then_e), Box::new(else_e))
            }
            // 91..=100 => Self::While(Expr::arbitrary_bool(g), Box::new(Self::generate_stmnts(g))),
            _ => Self::Skip,
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

fn clone_sets() -> (HashSet<String>, HashSet<String>) {
    (STORE.lock().unwrap().clone(), HEAP.lock().unwrap().clone())
}

fn restore_sets(set: &(HashSet<String>, HashSet<String>)) {
    STORE.lock().unwrap().clone_from(&set.0);
    HEAP.lock().unwrap().clone_from(&set.1);
}

// A cleaner to read string
fn arbitrary_ident(g: &mut Gen, store: bool) -> String {
    let mut s = String::new();
    // Occasionally use a random reference

    let mut i = u8::arbitrary(g) % 5 + 4;
    while i > 0 {
        // Just letters
        s.push(char::from(b'a' + u8::arbitrary(g) % 26));
        i -= 1;
    }
    if store {
        STORE.lock().unwrap().insert(s.clone());
    } else {
        HEAP.lock().unwrap().insert(s.clone());
    }
    s
}

fn random_store(g: &mut Gen) -> Option<String> {
    Some((*g.choose(&STORE.lock().unwrap().iter().collect::<Vec<_>>())?).to_string())
}

fn random_heap(g: &mut Gen) -> Option<String> {
    Some((*g.choose(&HEAP.lock().unwrap().iter().collect::<Vec<_>>())?).to_string())
}

pub fn quick_check(stmnt: Statement) -> TestResult {
    let typecheck = typecheck(&stmnt);
    let evaluated = eval_program(&stmnt);

    if typecheck.is_err() {
        println!(
            "{:?} typecheck error on {:?}\n",
            typecheck.unwrap_err(),
            stmnt
        );
        TestResult::failed()
    } else if evaluated.is_err() {
        println!(
            "{:?} evaluation error on {:?}\n",
            evaluated.unwrap_err(),
            stmnt
        );
        TestResult::failed()
    } else {
        println!("typecheck and evaluation pass on: {:?}\n", stmnt);
        TestResult::passed()
    }

}
