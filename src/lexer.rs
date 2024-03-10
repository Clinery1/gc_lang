use logos::Logos;


#[derive(Debug, Copy, Clone, PartialEq, Logos)]
#[logos(skip "\r")]
#[logos(skip "[ \t]*//[^\n]*")]
pub enum Token<'a> {
    #[token("let", |_|Keyword::Let)]
    #[token("mut", |_|Keyword::Mut)]
    #[token("set", |_|Keyword::Set)]
    #[token("func", |_|Keyword::Func)]
    #[token("proc", |_|Keyword::Proc)]
    #[token("type", |_|Keyword::Type)]
    #[token("disown", |_|Keyword::Disown)]
    #[token("scope", |_|Keyword::Scope)]
    #[token("if", |_|Keyword::If)]
    #[token("else", |_|Keyword::Else)]
    #[token("cond", |_|Keyword::Cond)]
    #[token("and", |_|Keyword::And)]
    #[token("or", |_|Keyword::Or)]
    #[token("return", |_|Keyword::Return)]
    Keyword(Keyword),

    #[regex("[A-Za-z_][A-Za-z0-9_]*")]
    Word(&'a str),
    #[regex("[0-9][0-9_]*")]
    Number(&'a str),
    #[regex("\"[^\"]*\"")]
    String(&'a str),

    // Enclosing punctuation
    #[token("{")]
    CurlyStart,
    #[token("}")]
    CurlyEnd,
    #[token("[")]
    SquareStart,
    #[token("]")]
    SquareEnd,
    #[token("(")]
    ParenStart,
    #[token(")")]
    ParenEnd,

    // Misc punctuation
    #[token(",")]
    #[regex(",[ \t\r\n]+")]
    Comma,
    #[token("~")]
    Tilde,
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,
    #[token("=>")]
    FatArrow,

    // Arithmetic
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("&")]
    And,
    #[token("|")]
    Or,
    #[token("^")]
    Xor,
    #[token("!")]
    Not,

    // Comparison
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,

    // Misc.
    #[token(".")]
    FieldIndex,

    // Whitespace
    #[regex("[ \t]*[\n\r]+")]
    Newline,
    #[regex("[ \t]+", |s|s.slice().len())]
    Whitespace(usize),

    EOF,
}
impl<'a> parser_helper::Token for Token<'a> {
    fn eof()->Self {Self::EOF}
}

#[derive(Debug, Copy, Clone, PartialEq, Logos)]
pub enum Keyword {
    Let,
    Mut,
    Set,
    Func,
    Proc,
    Type,
    Disown,
    Scope,
    If,
    Else,
    Cond,
    And,
    Or,
    Return,
}
