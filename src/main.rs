use check::quick_check_evaluator;
use syntax::Statement;

#[allow(unused)]
mod check;
pub mod error;
pub mod evaluator;
pub mod syntax;

#[macro_use]
extern crate lazy_static;

fn main() {
    quickcheck::quickcheck(quick_check_evaluator as fn(Statement) -> bool);
}