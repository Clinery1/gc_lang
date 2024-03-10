use parser_helper::{
    LogosWrapper,
    LookaheadLexer,
    SimpleError,
};
use misc_utils::stack::Stack;
use logos::Logos;
use std::borrow::Cow;
use crate::{
    lexer::*,
    StringInterner,
    Index,
};
use Keyword::*;

pub use tree::*;


mod tree;


pub type ParseResult<T> = Result<T, SimpleError<Cow<'static, str>>>;


pub struct Parser<'a> {
    pub interner: StringInterner<'a>,

    inner: LookaheadLexer<2, Token<'a>, LogosWrapper<'a, Token<'a>>, ()>,
    ws_stack: Stack<usize>,
}
impl<'a> Parser<'a> {
    pub fn new(source: &'a str)->Self {
        let l = LookaheadLexer::new(LogosWrapper(Token::lexer(source)), ());

        return Parser{
            inner: l,
            interner: StringInterner::new(),
            ws_stack: Stack::new(),
        };
    }

    fn indent(&mut self)->ParseResult<usize> {
        match self.next() {
            Token::Whitespace(count)=>Ok(count),
            _=>self.error("Expected indent"),
        }
    }

    fn try_indent(&mut self, amt: usize)->ParseResult<()> {
        let ret = match self.peek(0) {
            Token::Whitespace(count)=>if count == amt {
                self.next();
                Ok(())
            } else {
                self.error("Insufficient indentation")
            },
            _=>self.error("Not whitespace"),
        };

        return ret;
    }

    fn error<T, S: Into<Cow<'static, str>>>(&self, msg: S)->ParseResult<T> {
        Err(self.inner.error(msg.into()))
    }

    fn next(&mut self)->Token<'a> {
        self.inner.take_token()
    }

    fn peek(&mut self, i: usize)->Token<'a> {
        *self.inner.lookahead(i)
    }

    fn skip_ws(&mut self) {
        while let Token::Whitespace(_) = self.peek(0) {
            self.next();
        }
    }

    fn skip_nl(&mut self) {
        while let Token::Newline = self.peek(0) {
            self.next();
        }
    }

    fn ws(&mut self)->ParseResult<()> {
        match self.next() {
            Token::Whitespace(_)=>{
                self.skip_ws();
                Ok(())
            },
            _=>self.error("Expected whitespace"),
        }
    }

    fn eol(&mut self)->ParseResult<()> {
        let ret = match self.peek(0) {
            Token::Newline|Token::Semicolon|Token::EOF=>{
                self.next();
                Ok(())
            },
            _=>self.error("Expected `;` or EOL"),
        };

        while self.peek(0) == Token::Newline {
            self.next();
        }

        return ret;
    }

    fn match_token(&mut self, t: Token)->ParseResult<()> {
        if self.next() == t {
            Ok(())
        } else {
            self.error("Unexpected token")
        }
    }

    fn try_match(&mut self, t: Token)->bool {
        if self.peek(0) == t {
            self.next();
            true
        } else {
            false
        }
    }

    fn intern(&mut self, word: &'a str)->Index {
        self.interner.intern(word)
    }

    fn intern_string(&mut self, s: &'a str)->Index {
        let trimed = &s[1..s.len() - 1];
        self.intern(trimed)
    }

    fn word(&mut self)->ParseResult<Index> {
        match self.next() {
            Token::Word(w)=>Ok(self.intern(w)),
            _=>self.error("Expected word"),
        }
    }

    fn parse_num(&mut self, num_str: &'a str)->ParseResult<i64> {
        if let Ok(num) = num_str.parse::<i64>() {
            Ok(num)
        } else {
            self.error("Error parsing number")
        }
    }

    pub fn parse_file(&mut self)->ParseResult<Vec<Stmt>> {
        let mut stmts = Vec::new();
        self.ws_stack.push(0);

        while self.peek(0) != Token::EOF {
            stmts.push(self.parse_stmt()?);
        }

        return Ok(stmts);
    }

    pub fn parse_stmt(&mut self)->ParseResult<Stmt> {
        self.skip_nl();
        match self.peek(0) {
            Token::Keyword(Set)=>self.parse_var_set(),
            Token::Keyword(Let)=>self.parse_var_def(),
            Token::Keyword(Proc|Func)=>self.parse_function(),
            Token::Keyword(Scope)=>self.parse_scope(),
            Token::Keyword(Disown)=>self.parse_disown(),
            Token::Keyword(If)=>self.parse_if_else(),
            Token::Keyword(Cond)=>self.parse_cond(),
            Token::Keyword(Return)=>self.parse_return(),

            Token::Whitespace(_)=>self.error("Internal error: Unexpected indent"),
            _=>{
                let ret = self.parse_expr(0).map(Stmt::Expr)?;
                self.skip_ws();
                self.eol()?;
                Ok(ret)
            },
        }
    }

    fn parse_return(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Return))?;
        self.ws()?;

        if self.try_match(Token::Newline) {
            return Ok(Stmt::Return(None));
        } else {
            return self.parse_expr(0)
                .map(Option::Some)
                .map(Stmt::Return);
        }
    }

    fn parse_cond(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Cond))?;
        self.match_token(Token::Newline)?;
        self.skip_nl();

        let mut conditions = Vec::new();
        let mut actions = Vec::new();

        let current_indent = *self.ws_stack.last();
        let indent;
        match self.peek(0) {
            Token::Whitespace(amt)=>{
                if amt <= current_indent {
                    return self.error("Expected indented block");
                }

                indent = amt;
            },
            _=>return self.error("Expected indent"),
        }
        self.ws_stack.push(indent);

        loop {
            match self.peek(0) {
                Token::Whitespace(amt)=>{
                    if amt < indent {
                        break;
                    }
                    if amt > indent {
                        return self.error("Unexpected indent");
                    }
                    self.indent()?;
                },
                _=>break,
            }

            conditions.push(self.parse_expr(0)?);

            self.skip_ws();
            self.match_token(Token::FatArrow)?;
            self.skip_ws();

            match self.peek(0) {
                Token::Keyword(Scope)=>{
                    self.next();
                    self.match_token(Token::Newline)?;
                    self.skip_nl();
                    actions.push(ConditionalAction::Scope(self.parse_block()?));
                },
                _=>{
                    actions.push(ConditionalAction::Expr(self.parse_expr(0)?));
                    // self.match_token(Token::Newline)?;
                    self.skip_nl();
                },
            }
        }

        self.ws_stack.pop();

        return Ok(Stmt::Conditional {
            conditions,
            actions,
        });
    }

    fn parse_if_else(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(If))?;
        self.ws()?;

        let condition = self.parse_expr(0)?;
        self.match_token(Token::Newline)?;
        self.skip_nl();

        let block = self.parse_block()?;

        let mut default = None;

        let current_indent = *self.ws_stack.last();
        match (self.peek(0), self.peek(1)) {
            (Token::Whitespace(amt), Token::Keyword(Else))=>{
                if amt == current_indent {
                    self.indent()?;
                    self.next();
                    self.match_token(Token::Newline)?;
                    self.skip_nl();

                    default = Some(self.parse_block()?);
                }
            },
            _=>{},
        }

        return Ok(Stmt::IfElse {
            condition,
            block,
            default,
        });
    }

    fn parse_disown(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Disown))?;
        self.ws()?;

        let expr = self.parse_expr(0)?;

        self.eol()?;

        return Ok(Stmt::Disown(expr));
    }

    fn parse_scope(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Scope))?;
        self.skip_ws();
        self.match_token(Token::Newline)?;
        self.skip_nl();

        return self.parse_block().map(Stmt::Scope);
    }

    fn parse_var_def(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Let))?;
        self.ws()?;

        let mutable = self.try_match(Token::Keyword(Mut));
        if mutable {self.ws()?}

        let name = self.word()?;
        self.skip_ws();

        let data = if self.try_match(Token::Assign) {
            self.skip_ws();
            Some(self.parse_expr(0)?)
        } else {
            None
        };

        self.eol()?;

        return Ok(Stmt::VarDef {
            mutable,
            name,
            data,
        });
    }

    fn parse_var_set(&mut self)->ParseResult<Stmt> {
        self.match_token(Token::Keyword(Set))?;
        self.ws()?;

        let name = self.word()?;
        self.skip_ws();

        self.match_token(Token::Assign)?;
        self.skip_ws();

        let data = self.parse_expr(0)?;

        self.eol()?;

        return Ok(Stmt::VarSet {name, data});
    }

    fn parse_function(&mut self)->ParseResult<Stmt> {
        let is_proc = match self.next() {
            Token::Keyword(Proc)=>true,
            Token::Keyword(Func)=>false,
            _=>unreachable!("Function keyword"),
        };

        self.ws()?;

        let name = self.word()?;

        self.skip_ws();

        let mut pattern = self.parse_pattern()?;
        self.match_token(Token::Newline)?;
        self.skip_nl();

        let block = self.parse_block()?;

        return Ok(Stmt::FunctionDef {
            is_proc,
            name,
            pattern,
            block,
        });
    }

    fn parse_block(&mut self)->ParseResult<Block> {
        let mut stmts = Vec::new();
        let mut indent = 0;

        while self.peek(0) != Token::EOF {
            self.skip_nl();
            if indent == 0 {
                let last_indent = *self.ws_stack.last();
                match self.peek(0) {
                    Token::Whitespace(amt)=>{
                        if amt <= last_indent {
                            self.next();
                            return self.error("Expected indented block");
                        }

                        indent = self.indent()?;
                        self.ws_stack.push(indent);
                    },
                    _=>{
                        self.next();
                        return self.error("Expected indented block");
                    },
                }
            } else if self.try_indent(indent).is_err() {
                break;
            }

            stmts.push(self.parse_stmt()?);
        }

        self.ws_stack.pop();

        return Ok(Block(stmts));
    }

    pub fn parse_expr(&mut self, min_prec: u8)->ParseResult<Expr> {
        let mut ret = match self.peek(0) {
            Token::Mul=>{
                self.next();
                let inner = self.parse_expr(min_prec)?;
                Expr::Deref(Box::new(inner))
            },
            Token::And=>{
                self.next();
                let inner = self.parse_expr(min_prec)?;
                Expr::Borrow(Box::new(inner))
            },
            Token::ParenStart=>{
                self.next();
                let mut ret = None;

                loop {
                    self.skip_ws();

                    match self.peek(0) {
                        Token::ParenEnd=>{
                            self.next();
                            break;
                        },
                        _=>{},
                    }

                    self.skip_ws();
                    // If we are already a tuple, then add another item. Otherwise create a tuple.
                    match &mut ret {
                        Some(Expr::Group(items))=>items.push(self.parse_expr(0)?),
                        Some(_)=>ret = Some(Expr::Group(vec![ret.unwrap(), self.parse_expr(0)?])),
                        None=>ret = Some(self.parse_expr(0)?),
                    }

                    // Check for paren end or comma to start a list or end it.
                    match self.next() {
                        Token::Comma=>{},
                        Token::ParenEnd=>break,
                        _=>return self.error("Expected `,` or `)` in group"),
                    }
                }

                ret.unwrap_or(Expr::Group(Vec::new()))
            },
            _=>self.parse_expr_terminal()?,
        };

        loop {
            // Get the token's precedence
            let l_prec;
            let r_prec;
            match self.peek(0) {
                // If we have a whitespace, then check if there is a another operator after it.
                Token::Whitespace(_)=>{
                    let prec;
                    let peek = self.peek(1);
                    if let Some(p) = Self::infix_prec(peek) {
                        // If there is an operator after the whitespace, then skip it and get the
                        // next operator's precedence.
                        self.skip_ws();
                        prec = p;
                    } else if self.is_token_expr_start(peek) {
                        // If the token after the whitespace is an expr terminal or prefix
                        // operator, then get the whitespace's precedence
                        prec = Self::infix_prec(Token::Whitespace(0)).unwrap();
                    } else {
                        // Otherwise, it isn't an expr, so break the loop.
                        break;
                    }
                    l_prec = prec.0;
                    r_prec = prec.1;
                }
                // For every other token, check it.
                t=>if let Some((l, r)) = Self::infix_prec(t) {
                    l_prec = l;
                    r_prec = r;
                } else {break},
            }

            // If the precedence is too low, then stop the loop.
            if l_prec < min_prec {
                break;
            }

            // Get the operator
            let op = Self::infix_op(self.next());
            self.skip_ws();

            // Parse the right side and wrap the expression.
            ret = Expr::Operation {
                op,
                left: Box::new(ret),
                right: Box::new(self.parse_expr(r_prec)?),
            };
        }

        // postfix operations here
        while let Some((l_prec, r_prec)) = Self::postfix_prec(self.peek(0)) {
            if l_prec < min_prec {
                break;
            }

            match self.next() {
                Token::FieldIndex=>{
                    ret = Expr::Field {
                        left: Box::new(ret),
                        name: self.word()?,
                    };
                },
                tok=>{
                    ret = Expr::Operation {
                        op: Self::postfix_op(tok),
                        left: Box::new(ret),
                        right: Box::new(self.parse_expr(r_prec)?),
                    };
                },
            }
        }

        return Ok(ret);
    }

    fn is_token_expr_start(&self, token: Token)->bool {
        use Token::*;
        match token {
            Word(_)|Number(_)|String(_)|Mul|And|ParenStart=>true,
            _=>false,
        }
    }

    fn postfix_op(token: Token)->Operator {
        match token {
            _=>unreachable!("Postfix operator"),
        }
    }

    fn postfix_prec(token: Token)->Option<(u8, u8)> {
        match token {
            Token::FieldIndex=>Some((8, 9)),
            _=>None,
        }
    }

    fn infix_op(token: Token)->Operator {
        match token {
            Token::Equal=>Operator::Equal,
            Token::NotEqual=>Operator::NotEqual,
            Token::Less=>Operator::Less,
            Token::LessEqual=>Operator::LessEqual,
            Token::Greater=>Operator::Greater,
            Token::GreaterEqual=>Operator::GreaterEqual,
            Token::Add=>Operator::Add,
            Token::Sub=>Operator::Sub,
            Token::Mul=>Operator::Mul,
            Token::Div=>Operator::Div,
            Token::And=>Operator::And,
            Token::Or=>Operator::Or,
            Token::Xor=>Operator::Xor,
            Token::Whitespace(_)=>Operator::Apply,
            Token::Keyword(And)=>Operator::LogicAnd,
            Token::Keyword(Or)=>Operator::LogicOr,
            _=>unreachable!("Infix operator"),
        }
    }

    fn infix_prec(token: Token)->Option<(u8, u8)> {
        match token {
            Token::Keyword(And|Or)=>Some((0, 1)),
            Token::Equal|
                Token::NotEqual|
                Token::Less|
                Token::LessEqual|
                Token::Greater|
                Token::GreaterEqual=>Some((2,3)),
            Token::Whitespace(_)=>Some((5,4)),      // function application is left-associative
            Token::Add|Token::Sub=>Some((6,7)),
            Token::Mul|Token::Div=>Some((8,9)),
            Token::And|Token::Or|Token::Xor=>Some((10,11)),
            _=>None
        }
    }

    fn parse_expr_terminal(&mut self)->ParseResult<Expr> {
        match self.next() {
            Token::Number(num_str)=>Ok(Expr::Number(self.parse_num(num_str)?)),
            Token::Word("None")=>Ok(Expr::None),
            Token::Word(word)=>Ok(Expr::Var(self.intern(word))),
            Token::String(s)=>Ok(Expr::String(self.intern_string(s))),
            _=>self.error("Expected `expr`"),
        }
    }

    pub fn parse_pattern(&mut self)->ParseResult<Pattern> {
        Ok(match self.next() {
            Token::ParenStart=>{
                let mut items = Vec::new();

                loop {
                    self.skip_ws();

                    match self.peek(0) {
                        Token::ParenEnd=>{
                            self.next();
                            break;
                        },
                        _=>{},
                    }

                    items.push(self.parse_pattern()?);

                    self.skip_ws();

                    match self.next() {
                        Token::ParenEnd=>break,
                        Token::Comma=>{},
                        _=>return self.error("Expected `)` or `,` in pattern"),
                    }
                }

                Pattern::Group(items)
            },
            Token::Word("None")=>Pattern::None,
            Token::Word(w)=>Pattern::Name(self.intern(w)),
            Token::Number(n)=>Pattern::Number(self.parse_num(n)?),
            _=>return self.error("Unexpected token in pattern"),
        })
    }
}
