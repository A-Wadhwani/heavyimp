use check_eval::quick_check;
use quickcheck::{TestResult, Gen};
use syntax::Statement;

mod check_eval;
pub mod error;
pub mod evaluator;
pub mod syntax;
pub mod typechecker;

#[macro_use]
extern crate lazy_static;

fn main() {
    // Check if the evaluator does not throw an error, given that the type-checker passes
    quickcheck::QuickCheck::new()
        .tests(500)
        .max_tests(5000000)
        .min_tests_passed(500)
        .gen(Gen::new(30))
        .quickcheck(quick_check as fn(Statement) -> TestResult);
}
