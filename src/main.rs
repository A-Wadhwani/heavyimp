use check_eval::quick_check_evaluator;
use quickcheck::{TestResult, Gen};
use syntax::Statement;

mod check_eval;
pub mod error;
pub mod evaluator;
pub mod syntax;

#[macro_use]
extern crate lazy_static;

fn main() {
    // Check if the evaluator does not throw an error, given that the type-checker passes
    quickcheck::QuickCheck::new()
        .max_tests(1000000)
        .min_tests_passed(300)
        .gen(Gen::new(30))
        .quickcheck(quick_check_evaluator as fn(Statement) -> TestResult);
}
