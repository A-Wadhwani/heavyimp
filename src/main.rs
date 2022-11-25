use check_eval::{check_correct, check_eval_type, check_type_eval, toggle_random};
use quickcheck::{Gen, TestResult};
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
    toggle_random(true);
    quickcheck::QuickCheck::new()
        .min_tests_passed(1000)
        .tests(4000)
        .max_tests(100000)
        .gen(Gen::new(15))
        .quickcheck(check_type_eval as fn(Statement) -> TestResult);
    // Check if the evaluator and type-checker do not throw errors on correct programs
    toggle_random(false);
    quickcheck::QuickCheck::new()
        .tests(30000)
        .max_tests(30000)
        .gen(Gen::new(65))
        .quickcheck(check_correct as fn(Statement) -> TestResult);
    // Check if the type-checker *does* throw an error, given that the evaluator fails
    toggle_random(true);
    quickcheck::QuickCheck::new()
        .min_tests_passed(20000)
        .tests(30000)
        .max_tests(30000)
        .gen(Gen::new(65))
        .quickcheck(check_eval_type as fn(Statement) -> TestResult);
}

#[cfg(test)]
mod tests {
    use crate::main;

    #[test]
    fn quick_check() {
        main();
    }
}
