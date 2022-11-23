use std::{collections::HashSet, sync::Mutex};

use quickcheck::Arbitrary;
use syntax::{Constant, Constant::*, Expr, Statement};

pub mod error;
#[allow(unused)]
pub mod evaluator;
pub mod syntax;

#[macro_use]
extern crate lazy_static;

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
}

impl Arbitrary for Expr {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 10 {
            0 => Expr::StoreRead(random_reference(g)),
            1 => Expr::HeapRead(random_reference(g)),
            2..=5 => Expr::Constant(Constant::arbitrary(g)),
            6 => Expr::NatAdd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            7 => Expr::NatLeq(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            8 => Expr::BoolAnd(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g))),
            9 => Expr::BoolNot(Box::new(Self::arbitrary(g))),
            _ => unreachable!(),
        }
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        quickcheck::empty_shrinker()
    }
}

impl Arbitrary for Statement {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        match u8::arbitrary(g) % 100 + 1 {
            1..=15 => Statement::StoreAssign(arbitrary_ident(g), Expr::arbitrary(g)),
            16..=30 => Statement::HeapNew(arbitrary_ident(g), Expr::arbitrary(g)),
            31..=35 => Statement::HeapUpdate(arbitrary_ident(g), Expr::arbitrary(g)),
            36..=40 => Statement::HeapAlias(arbitrary_ident(g), random_reference(g)),
            41..=60 => {
                Statement::Sequence(Box::new(Self::arbitrary(g)), Box::new(Self::arbitrary(g)))
            }
            61..=80 => Statement::Conditional(
                Expr::arbitrary(g),
                Box::new(Self::arbitrary(g)),
                Box::new(Self::arbitrary(g)),
            ),
            81..=90 => Statement::While(Expr::arbitrary(g), Box::new(Self::arbitrary(g))),
            91..=100 => Statement::Skip,
            _ => unreachable!(),
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

fn random_reference(g: &mut quickcheck::Gen) -> String {
    g.choose(&NAMES.lock().unwrap().iter().collect::<Vec<_>>())
        .map_or("x".into(), |s| s.to_string())
}

fn quick_check_evaluator(stmnt: Statement) -> bool {
    println!("Running quickcheck on: {:?}\n", stmnt);
    let val = evaluator::eval_program(&stmnt).is_ok();
    NAMES.lock().unwrap().clear();
    val
}

fn main() {
    // After implementing the type-checker, the idea is to ensure that every statement that type checks also evaluates (or times out).
    quickcheck::quickcheck(quick_check_evaluator as fn(Statement) -> bool);
}
