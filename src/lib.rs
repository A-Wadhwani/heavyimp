use wasm_bindgen::prelude::*;

use crate::error::ImpParseError;

pub mod error;
pub mod evaluator;
pub mod parser;
pub mod syntax;
mod test;
pub mod typechecker;

#[wasm_bindgen(getter_with_clone)]
pub struct Success {
    pub parsed: String,
    pub evaluated: String,
}

#[wasm_bindgen]
pub fn run_program(source: &str) -> Result<Success, String> {
    let parsed = parser::parse(&source).map_err(|e| {
        let ImpParseError::Other(s) = e;
        format!("Parser Error:\n{}", s)
    })?;

    if let Err(e) = typechecker::typecheck(&parsed) {
        return Err(format!("Typecheck Error: {:?}", e));
    }

    let evaluated = evaluator::eval_program(&parsed);
    match evaluated {
        Ok(e) => Ok(Success {
            parsed: format!("{:?}", parsed),
            evaluated: format!("{:?}", e),
        }),
        Err(e) => Err(format!("Evaluation Error: {:?}", e).as_str().into()),
    }
}
