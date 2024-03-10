// lifetime checking


use std::{
    collections::HashMap,
    rc::Rc,
};
use fnv::FnvHashMap;
use crate::{
    Index,
    Name,
};

pub use crate::parser::{
    Operator,
    Pattern,
};


#[derive(Debug)]
pub enum Stmt {
    VarDef(VarIndex),
    VarSet {
        name: Name,
        data: ExprIndex,
        var: VarIndex
    },
    IfElse {
        condition: ExprIndex,
        block: Block,
        else_block: Option<Block>,
        last: StmtIndex,
    },
    Conditional {
        conditions: Vec<ExprIndex>,
        actions: Vec<ConditionalAction>,
        last: StmtIndex,
    },
    Disown(ExprIndex),
    Expr(ExprIndex),
    Return(Option<ExprIndex>),

    JumpTo(StmtIndex),
    Skip,
}

#[derive(Debug)]
pub enum Expr {
    /// <expr> <op> <expr>
    Operation {
        left: ExprIndex,
        right: ExprIndex,
        op: Operator,
    },
    /// <expr> # <word>
    Field {
        left: ExprIndex,
        name: Index,
    },
    /// '[' <expr> (',' <expr>)+ ','? ']'
    Group(Vec<ExprIndex>),
    RawVar(Name),
    Number(i64),
    String(Index),
    Borrow(ExprIndex),
    Deref(ExprIndex),
    None,

    Var(VarIndex),
    Function(FunctionIndex),
    /// Used to convey an optimized-out expression
    Skip,
}

#[derive(Debug)]
pub enum ConditionalAction {
    Expr(ExprIndex),
    Scope(Block),
}

#[derive(Debug)]
pub enum Type {
    Ref(Box<Self>),
    Tuple(Vec<Self>),
    String,
    Number,
    Undetermined,
}

#[derive(Debug)]
pub enum MemoryLocation {
    Stack(usize),
    Heap,
    Undetermined,
}


/// The root stmt has a patch index of 0.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct StmtIndex {
    pub root: usize,
    pub patch: usize,
}
impl StmtIndex {
    #[inline]
    pub const fn invalid()->Self {
        StmtIndex {
            root: usize::MAX,
            patch: 0,
        }
    }
}

/// The root stmt has a patch index of 0.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ExprIndex {
    pub root: usize,
    pub patch: usize,
}
impl ExprIndex {
    #[inline]
    pub const fn invalid()->Self {
        ExprIndex {
            root: usize::MAX,
            patch: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct ScopeIndex(pub usize);
impl ScopeIndex {
    #[inline]
    pub const fn invalid()->Self {
        ScopeIndex(usize::MAX)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct FunctionIndex(pub usize);
impl FunctionIndex {
    #[inline]
    pub const fn invalid()->Self {
        FunctionIndex(usize::MAX)
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct VarIndex(pub usize);
impl VarIndex {
    #[inline]
    pub const fn invalid()->Self {
        VarIndex(usize::MAX)
    }
}

#[derive(Debug)]
pub struct Scope {
    pub first: StmtIndex,
    pub last: StmtIndex,

    pub stack_slots: usize,

    /// A map of `name -> var_list` where `var_list` is a list of var data. Multiple vars with the
    /// same name can exist in the same scope if one is disowned or moved (WIP)
    pub vars: FnvHashMap<Name, Vec<VarIndex>>,
    /// A map of `name -> function_list` where `function_list` is a map of `pattern -> function`
    pub functions: FnvHashMap<Name, HashMap<Rc<Pattern>, FunctionIndex>>,

    pub scopes: Vec<ScopeIndex>,
}

#[derive(Debug)]
pub struct VarMetadata {
    pub in_scope: ScopeIndex,

    pub definition: StmtIndex,
    pub init: Option<ExprIndex>,
    pub disown: Option<StmtIndex>,

    pub data_type: Type,

    pub borrows: Vec<StmtIndex>,
    pub uses: Vec<StmtIndex>,
    pub derefs: Vec<StmtIndex>,
    pub assigns: Vec<StmtIndex>,

    pub mem_loc: MemoryLocation,

    pub mutable: bool,
    pub name: Name,
}

#[derive(Debug)]
pub struct File {
    pub stmts: Vec<Stmt>,
    pub patch_stmts: FnvHashMap<usize, Vec<Stmt>>,

    pub exprs: Vec<Expr>,
    pub patch_exprs: FnvHashMap<usize, Vec<Expr>>,

    pub scopes: Vec<Scope>,

    pub functions: Vec<FunctionDef>,

    pub vars: Vec<VarMetadata>,

    pub root_scope: ScopeIndex,
}
impl File {
    pub fn new()->Self {
        File {
            stmts: Vec::new(),
            patch_stmts: FnvHashMap::default(),
            exprs: Vec::new(),
            patch_exprs: FnvHashMap::default(),
            scopes: Vec::new(),
            functions: Vec::new(),
            vars: Vec::new(),
            root_scope: ScopeIndex(0),
        }
    }

    pub fn add_var(&mut self, var: VarMetadata)->VarIndex {
        let index = VarIndex(self.vars.len());
        self.vars.push(var);
        index
    }

    pub fn add_function(&mut self, function: FunctionDef)->FunctionIndex {
        let index = FunctionIndex(self.functions.len());
        self.functions.push(function);
        index
    }

    pub fn add_scope(&mut self, scope: Scope)->ScopeIndex {
        let index = ScopeIndex(self.scopes.len());
        self.scopes.push(scope);
        index
    }

    pub fn get_var(&self, var: VarIndex)->&VarMetadata {
        &self.vars[var.0]
    }

    pub fn get_function(&self, function: FunctionIndex)->&FunctionDef {
        &self.functions[function.0]
    }

    pub fn get_scope(&self, scope: ScopeIndex)->&Scope {
        &self.scopes[scope.0]
    }

    pub fn get_mut_var(&mut self, var: VarIndex)->&mut VarMetadata {
        &mut self.vars[var.0]
    }

    pub fn get_mut_function(&mut self, function: FunctionIndex)->&mut FunctionDef {
        &mut self.functions[function.0]
    }

    pub fn get_mut_scope(&mut self, scope: ScopeIndex)->&mut Scope {
        &mut self.scopes[scope.0]
    }

    pub fn add_stmt(&mut self, stmt: Stmt)->StmtIndex {
        let index = StmtIndex {
            root: self.stmts.len(),
            patch: 0,
        };
        self.stmts.push(stmt);
        index
    }

    pub fn patch_stmt(&mut self, patch: Stmt, location: StmtIndex)->StmtIndex {
        let entry = self.patch_stmts.entry(location.root).or_default();
        entry.push(patch);
        let index = StmtIndex {
            root: location.root,
            patch: entry.len(),
        };
        index
    }

    pub fn add_expr(&mut self, expr: Expr)->ExprIndex {
        let index = ExprIndex {
            root: self.exprs.len(),
            patch: 0,
        };
        self.exprs.push(expr);
        index
    }

    pub fn patch_expr(&mut self, patch: Expr, location: ExprIndex)->ExprIndex {
        let entry = self.patch_exprs.entry(location.root).or_default();
        entry.push(patch);
        let index = ExprIndex {
            root: location.root,
            patch: entry.len(),
        };
        index
    }

    pub fn get_stmt(&self, loc: StmtIndex)->&Stmt {
        if loc.patch == 0 {
            &self.stmts[loc.root]
        } else {
            &self.patch_stmts.get(&loc.root).unwrap()[loc.patch - 1]
        }
    }

    pub fn get_expr(&self, loc: ExprIndex)->&Expr {
        if loc.patch == 0 {
            &self.exprs[loc.root]
        } else {
            &self.patch_exprs.get(&loc.root).unwrap()[loc.patch - 1]
        }
    }

    pub fn get_mut_stmt(&mut self, loc: StmtIndex)->&mut Stmt {
        if loc.patch == 0 {
            &mut self.stmts[loc.root]
        } else {
            &mut self.patch_stmts.get_mut(&loc.root).unwrap()[loc.patch - 1]
        }
    }

    pub fn get_mut_expr(&mut self, loc: ExprIndex)->&mut Expr {
        if loc.patch == 0 {
            &mut self.exprs[loc.root]
        } else {
            &mut self.patch_exprs.get_mut(&loc.root).unwrap()[loc.patch - 1]
        }
    }
}

#[derive(Debug)]
pub struct FunctionDef {
    /// This determines `func` or `proc` status.
    pub is_proc: bool,
    pub name: Name,
    pub pattern: Rc<Pattern>,

    pub block: Block,
}

#[derive(Debug)]
pub struct Block {
    pub first: StmtIndex,
    pub last: StmtIndex,

    pub scope: ScopeIndex,
}
