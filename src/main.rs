//! For all of this code, I will be using `function` when referring to either a `func` or `proc`
//! piece of code. When the semantic meaning is more important, I will use `func` or `proc` in
//! backticks. All of the code that deals with `func`s and `proc`s will be the same, except for the
//! parser and any optimizers. Execution will still be the same for either.
//!
//! Memory is not manually managed. You can disown data, but never manually deallocate it (WIP).
//! Once you disown a variable or data, the compiler marks the data is disowned and will deallocate
//! it if there are no references.
//!
//! Procedures are things that have side effects and are like memory barriers for atomic operations.
//! Functions are repeatable things that are a function of their inputs, and can be reordered.
//!
//! For now, we have dynamic typing. Static typing will get added later.
//! Objects will by default be held in a stack. Any referenced objects will be automatically hoisted
//! to the heap for longevity.
//!
//! This language may add an effect system later, but for now, `proc` and `func` are treated the
//! same at execution time. I may add an automatic memoization ability to `func`s, but it is
//! unlikely.
//! If I do add an effect system, then it will be closely tied into the type system, and error
//! handling. Here is an annotated example:
//! ``` 
//! enum IOErrorKind {
//!     NotFound,
//!     PermissionDenied,
//!     // etc.
//! }
//!
//!
//! // This can be used to differentiate the different effects, and identify what effects a system
//! // may have.
//! effect type IOError = {
//!     kind: IOErrorKind,
//!     msg: Option(String),
//! }
//!
//! effect type Error = IOError | MemError
//!
//! // This is just an effect with no data. It is only useful to say "there is this effect"
//! effect type BlankEffect = {}
//! ```


use indexmap::IndexSet;
use logos::Logos;
use std::fs::read_to_string;


mod lexer;
mod parser;
mod mid_ast;


pub type Name = Index;


#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Index(usize);

pub struct StringInterner<'a> {
    strings: IndexSet<&'a str>,
}
impl<'a> StringInterner<'a> {
    /// Create a new StringInterner.
    pub fn new()->Self {
        StringInterner {
            strings: IndexSet::new(),
        }
    }

    /// Intern the string and return the index.
    pub fn intern(&mut self, s: &'a str)->Index {
        Index(self.strings.insert_full(s).0)
    }

    /// Returns the index of the given string
    pub fn get_index(&self, s: &'a str)->Option<Index> {
        self.strings.get_index_of(s).map(Index)
    }

    /// Returns the string with the given index. Panics if the index is invalid.
    pub fn get_string(&self, i: Index)->&'a str {
        self.strings.get_index(i.0).expect("Invalid index!")
    }
}


fn main() {
    let file = read_to_string("example").unwrap();
    for token in lexer::Token::lexer(&file) {
        dbg!(token).ok();
    }

    let mut parser = parser::Parser::new(&file);
    let res = dbg!(parser.parse_file());
    match res {
        Ok(items)=>{
            for item in &items {
                item.print(&parser.interner, 0);
                println!();
            }

            for (i, s) in parser.interner.strings.iter().enumerate() {
                println!("{i}: \"{s}\"");
            }

            dbg!(mid_ast::conversion::convert_parse_tree(items));
        },
        Err(e)=>{
            e.eprint_with_source(&file, "example");
        },
    }
}
