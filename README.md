# About
This project is a simple foray into making a garbage collected language. I am going to follow some
of Lox's ideas for the collector, but I will probably do it in a much safer, Rustier way.


# Inspiration
- Lox
- Rust
- Haskell
- Python
- That one language with pattern matching polymorphism that I can't remember
- The functional paradigm


# Plans
- Write a garbage collector and bytecode interpreter
- "Compile-time" checks for borrowing, variable initialization, function redeclaration, variable
    redeclaration, etc.
- Some optimizations like constant folding, dead code elimination, inlining of functions, loop
    unrolling (when I implement loops), etc.
- Loops: while, for, and forever including the standard break/continue control flow.


# Possibly familiar features
- Logical AND and OR are both control flow constructs, and short-circuit
- Standard arithmetic, bitwise, logical, and comparison operators
- `proc` for procedures and `func` for pure functions (WIP)
- Significant whitespace, but better than Python's implementation
- Pattern matching polymorphism
- Garbage collection
- Conditional blocks (basically a `switch` block, but made to look like a Rust `match` block)
- Not horrible error messages thanks to my library


# Possibly controversial features
- Significant whitespace
- Lack of curly brackets
- Function calls are a space
- Pattern matching for function parameters
- No else-if or elif blocks. Those are replaced with the `cond` (conditional) block
- Garbage collection
- Hand-written, probably buggy parser
