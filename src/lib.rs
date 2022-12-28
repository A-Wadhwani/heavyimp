use wasm_bindgen::prelude::*;

use crate::error::ImpParseError;

pub mod error;
pub mod evaluator;
pub mod parser;
pub mod syntax;
mod test;
pub mod typechecker;

#[wasm_bindgen]
pub fn run_str(source: &str) {
    let parsed = parser::parse(&source).unwrap_or_else(|e| {
        let ImpParseError::Other(s) = e;
        eprintln!("Parser Error:\n{}", s);
        std::process::exit(1);
    });

    println!("Parsed");
    println!("===============");
    println!("{:?}", &parsed);

    let typecheck = typechecker::typecheck(&parsed);
    match typecheck {
        Ok(_) => {
            println!("\nEvaluated");
            println!("===============");
            println!("{:?}", evaluator::eval_program(&parsed))
        }
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
