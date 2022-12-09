# Heapy Imp

A typechecker and interpreter for Heapy Imp

```lua
let x <- 0
let inc = 25

while *x < 100 do
    x <- *x + inc
end
```

## Implemented Features
- Parser
    - Implemented with the help of pest, and using a lua-like syntax
- Typechecker
    - Refer to `typing_rules.pdf`
- Interpreter
    - Produces a map from variables to values or locations on the heap, and an array of values on the heap.
    - To avoid infinite loops during quickcheck tests, the interpreter has a maximum number of iterations in a loop it can execute.
- Quickcheck tests
    - Randomly generates programs, and checks that they typecheck and evaluate correctly.

## Running and Testing
You can run programs through: `cargo run examples/<file>.imp`

There's also additional tests in the program, and in particular, there are quick-check tests to ensure the three following properties:
1. Programs that type-check won't have an evaluation error
2. Programs that have an evaluation error won't type-check
3. "Correct" programs will type-check and evaluate

The best way to run these is using `cargo test --release quick_check  -- --nocapture` (the `--nocapture` is important to see the output of the tests).
Additionally, to see the output from the quickcheck library, it's necessary to set the `RUST_LOG` environment variable to "quickcheck".