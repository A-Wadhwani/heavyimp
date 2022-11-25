mod test;
pub mod error;
pub mod evaluator;
pub mod parser;
pub mod syntax;
pub mod typechecker;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match &args[..] {
        [_, file] => {
            let source = std::fs::read_to_string(file).unwrap();
            run_str(&source);
        }
        _ => eprintln!("Expected 'cargo run <file>'"),
    }
}

fn run_str(source: &str) {
    let parsed = parser::parse(&source).unwrap();
    dbg!(&parsed);

    let typecheck = typechecker::typecheck(&parsed);
    match typecheck {
        Ok(_) => println!("{:?}", evaluator::eval_program(&parsed)),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
