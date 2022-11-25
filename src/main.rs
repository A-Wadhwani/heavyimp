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
        .min_tests_passed(500)
        .tests(4000)
        .max_tests(100000)
        .gen(Gen::new(20))
        .quickcheck(quick_check as fn(Statement) -> TestResult);
}

#[cfg(test)]
mod tests {
    use crate::main;

    #[test]
    fn quick_check() {
        main();
    }
}