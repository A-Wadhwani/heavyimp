use check_eval::quick_check_evaluator;
use quickcheck::TestResult;
use syntax::Statement;

mod check_eval;
pub mod error;
pub mod evaluator;
pub mod syntax;

#[macro_use]
extern crate lazy_static;

fn main() {
    // Check if the evaluator does not throw an error, given that the type-checker passes
    quickcheck::QuickCheck::new().quickcheck(quick_check_evaluator as fn(Statement) -> TestResult);
}
