// parser


use std::hash::{
    Hash,
    Hasher,
};
use crate::{
    Index,
    Name,
    StringInterner,
};


#[derive(Debug)]
pub enum Stmt {
    FunctionDef {
        is_proc: bool,
        name: Name,
        pattern: Pattern,
        block: Block,
    },
    VarDef {
        mutable: bool,
        name: Name,
        data: Option<Expr>,
    },
    VarSet {
        name: Name,
        data: Expr,
    },
    IfElse {
        condition: Expr,
        block: Block,
        default: Option<Block>,
    },
    Conditional {
        conditions: Vec<Expr>,
        actions: Vec<ConditionalAction>,
    },
    Scope(Block),
    Disown(Expr),
    Return(Option<Expr>),
    Expr(Expr),
}
impl Stmt {
    pub fn print(&self, interner: &StringInterner, indent: usize) {
        for _ in 0..indent {print!(" ")}
        match self {
            Stmt::Expr(expr)=>{
                expr.print(interner);
                println!();
            },
            Stmt::FunctionDef{is_proc, name, pattern, block}=>{
                if *is_proc {
                    print!("proc ");
                } else {
                    print!("func ");
                }

                print!("{} ", interner.get_string(*name));

                pattern.print(interner);

                println!();

                block.print(interner, indent + 4);
            },
            Stmt::VarDef{mutable, name, data}=>{
                print!("let ");
                if *mutable {print!("mut ")}

                print!("{}", interner.get_string(*name));

                if let Some(data) = data {
                    print!(" = ");
                    data.print(interner);
                }

                println!();
            },
            Stmt::VarSet{name, data}=>{
                print!("set {} = ", interner.get_string(*name));

                data.print(interner);

                println!();
            },
            Stmt::Scope(block)=>{
                println!("scope");
                block.print(interner, indent + 4);
            },
            Stmt::Disown(expr)=>{
                expr.print(interner);
                println!();
            },
            Stmt::IfElse{condition, block, default}=>{
                print!("if ");
                condition.print(interner);
                println!();

                block.print(interner, indent + 4);

                if let Some(else_block) = default {
                    for _ in 0..indent {print!(" ")}
                    println!("else");
                    else_block.print(interner, indent + 4);
                }
            },
            Stmt::Conditional{conditions, actions}=>{
                println!("cond");
                for (condition, block) in conditions.iter().zip(actions.iter()) {
                    for _ in 0..(indent + 4) {print!(" ")}
                    if condition.is_group() {
                        condition.print(interner);
                    } else {
                        print!("(");
                        condition.print(interner);
                        print!(")");
                    }
                    print!(" => ");
                    block.print(interner, indent + 8);
                }
            },
            Stmt::Return(opt_expr)=>{
                print!("return ");
                if let Some(expr) = opt_expr {
                    expr.print(interner);
                }
                println!();
            },
        }
    }
}

#[derive(Debug)]
pub enum ConditionalAction {
    Expr(Expr),
    Scope(Block),
}
impl ConditionalAction {
    pub fn print(&self, interner: &StringInterner, indent: usize) {
        match self {
            Self::Expr(expr)=>{
                expr.print(interner);
                println!();
            },
            Self::Scope(block)=>{
                println!("scope");
                block.print(interner, indent);
            },
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    /// <expr> <op> <expr>
    Operation {
        left: Box<Self>,
        right: Box<Self>,
        op: Operator,
    },
    /// <expr> # <word>
    Field {
        left: Box<Self>,
        name: Name,
    },
    /// '[' <expr> (',' <expr>)+ ','? ']'
    Group(Vec<Self>),
    Var(Name),
    Number(i64),
    String(Index),
    Borrow(Box<Self>),
    Deref(Box<Self>),
    None,
}
impl Expr {
    /// Checks if self is an enclosed group of data
    pub fn is_group(&self)->bool {
        match self {
            Self::None|
                Self::Group(_)|
                Self::String(_)|
                Self::Number(_)|
                Self::Field{..}|
                Self::Var(_)=>true,
            _=>false,
        }
    }

    pub fn print(&self, interner: &StringInterner) {
        match self {
            Expr::Operation{op,left,right}=>{
                if left.is_group() {
                    left.print(interner);
                } else {
                    print!("(");
                    left.print(interner);
                    print!(")");
                }
                op.print();
                if right.is_group() {
                    right.print(interner);
                } else {
                    print!("(");
                    right.print(interner);
                    print!(")");
                }
            },
            Expr::Field{left, name}=>{
                if left.is_group() {
                    left.print(interner);
                } else {
                    print!("(");
                    left.print(interner);
                    print!(")");
                }
                print!(".{}", interner.get_string(*name));
            },
            Expr::Var(name)=>print!("{}", interner.get_string(*name)),
            Expr::Number(n)=>print!("{n}"),
            Expr::String(s)=>print!("\"{}\"", interner.get_string(*s)),
            Expr::None=>print!("None"),
            Expr::Borrow(inner)=>{
                print!("&");
                inner.print(interner);
            },
            Expr::Deref(inner)=>{
                print!("*");
                inner.print(interner);
            },
            Expr::Group(list)=>{
                if list.len() == 0 {
                    print!("()");
                } else {
                    print!("(");
                    list[0].print(interner);
                    for i in &list[1..] {
                        print!(", ");
                        i.print(interner);
                    }
                    print!(")");
                }
            },
            // _=>todo!(),
        }
    }
}

#[derive(Debug)]
pub enum Operator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,

    // Bitwise/logic
    And,
    Or,
    Xor,

    // Comparisons
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logic
    LogicAnd,
    LogicOr,

    // Misc.
    Apply,
}
impl Operator {
    pub fn print(&self) {
        use Operator::*;
        match self {
            Add=>print!(" + "),
            Sub=>print!(" - "),
            Mul=>print!(" * "),
            Div=>print!(" / "),

            And=>print!(" & "),
            Or=>print!(" | "),
            Xor=>print!(" ^ "),

            Equal=>print!(" == "),
            NotEqual=>print!(" != "),
            Less=>print!(" < "),
            LessEqual=>print!(" <= "),
            Greater=>print!(" > "),
            GreaterEqual=>print!(" >= "),

            LogicAnd=>print!(" and "),
            LogicOr=>print!(" or "),

            Apply=>print!(" "),
        }
    }
}

/// The Hash and PartialEq implementation do not consider patterns of variant `Name` to be
/// different from each other regardless of contents. This means that we can simply hash the
/// pattern and figure out if there is something that fits it or not already.
#[derive(Debug, Eq)]
pub enum Pattern {
    Group(Vec<Self>),
    Name(Name),
    Number(i64),
    None,
}
impl Pattern {
    pub fn print(&self, interner: &StringInterner) {
        match self {
            Self::Group(items)=>{
                if items.len() == 0 {
                    print!("()");
                } else {
                    print!("(");
                    items[0].print(interner);

                    for item in &items[1..] {
                        print!(", ");
                        item.print(interner);
                    }
                    print!(")");
                }
            },
            Self::Name(n)=>print!("{}", interner.get_string(*n)),
            Self::Number(n)=>print!("{n}"),
            Self::None=>print!("None"),
        }
    }
}
impl Hash for Pattern {
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self {
            Self::Group(items)=>{
                h.write_u8(0);
                for item in items {
                    item.hash(h);
                }
                h.write_u8(1);
            },
            Self::Name(_)=>h.write_u8(2),
            Self::Number(n)=>{
                h.write_u8(3);
                h.write_i64(*n);
            },
            Self::None=>h.write_u8(4),
        }
    }
}
impl PartialEq for Pattern {
    fn eq(&self, o: &Self)->bool {
        use Pattern::*;
        match (self, o) {
            (Group(l), Group(r))=>l == r,
            (Name(_), Name(_))=>true,
            (Number(l), Number(r))=>l == r,
            (None, None)=>true,
            _=>false,
        }
    }
}


#[derive(Debug)]
pub struct Block(pub Vec<Stmt>);
impl Block {
    pub fn print(&self, interner: &StringInterner, indent: usize) {
        for stmt in self.0.iter() {
            stmt.print(interner, indent);
        }
    }
}
