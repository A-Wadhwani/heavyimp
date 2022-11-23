use check_eval::quick_check_evaluator;
use syntax::Statement;

#[allow(unused)]
mod check_eval;
pub mod error;
pub mod evaluator;
pub mod syntax;

#[macro_use]
extern crate lazy_static;

fn main() {
    quickcheck::quickcheck(quick_check_evaluator as fn(Statement) -> bool);
}