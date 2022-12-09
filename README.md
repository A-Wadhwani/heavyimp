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
    - Designed such that type inference is unnecessary
        - `let` always denotes a new binding
        - `=` assigns to the store
        - `<-` assigns to the heap
        - `*` reads from the heap. No pointer arithmetic is allowed, so it must always come before an identifier.
- Typechecker
    - Refer to `typing_rules.pdf`
- Interpreter
    - Produces a map from variables to values or locations on the heap, and an array of values on the heap.
    - To avoid infinite loops during quickcheck tests, the interpreter has a maximum number of iterations in a loop it can execute.
- Quickcheck tests
    - We have control over how many of the generated programs will be correct by first
    generating correct programs and then randomly messing with them.
    - Randomly generates programs, and checks that they typecheck and evaluate correctly.

## Important Note About Typing Rules

Conditionals allow variables from either conditional branch to leak if they have the same
type in each branch. The intersection sign in T-Conditional denotes variables that have the
same type in each scope.

The behavior for this example is interesting because of this:

```
let z <- 10
if *z < 10 then
    let x <- 5
    let a = 1
    let b <- 1
else
    let x <- 10
    let b = 1
    let a <- 1
fi
let y <- *x
```

Here, x has the same type in each branch, so the assignment on the last line succeeds. However,
if we attempt to assign to either b or a we get an `UnboundVariable` error from the typechecker,
since they have different types in each branch.

The evaluator doesn't know this, so the ending scope will still contain every variable for the
branch that was actually executed.

## Running and Testing
You can run programs through: `cargo run examples/<file>.imp`

There's also additional tests in the program, and in particular, there are quick-check tests to ensure the three following properties:
1. Programs that type-check won't have an evaluation error
2. Programs that have an evaluation error won't type-check
3. "Correct" programs will type-check and evaluate

The best way to run these is using `cargo test --release quick_check  -- --nocapture` (the `--nocapture` is important to see the output of the tests).
Additionally, to see the output from the quickcheck library, it's necessary to set the `RUST_LOG` environment variable to "quickcheck".
