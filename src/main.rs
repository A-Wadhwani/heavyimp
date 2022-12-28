use heavyimp::run_str;

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
