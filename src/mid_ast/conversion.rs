use std::{
    collections::HashMap,
    rc::Rc,
};
use fnv::FnvHashMap;
use crate::{
    parser::{
        Stmt as PStmt,
        Expr as PExpr,
        ConditionalAction as PCondAct,
        Block as PBlock,
    },
    Index,
    Name,
};
use super::tree::*;


struct FileConversion {
    file: File,

    raw_func_queue: Vec<RawFunction>,
}
impl FileConversion {
    fn convert(stmts: Vec<PStmt>)->File {
        let mut this = FileConversion {
            file: File::new(),
            raw_func_queue: Vec::new(),
        };

        this.file.root_scope = this.convert_block(PBlock(stmts)).scope;

        // convert all of the functions in some random order
        while let Some(raw_function) = this.raw_func_queue.pop() {
            this.convert_function(raw_function);
        }

        return this.file;
    }

    fn next_stmt_index(&self)->StmtIndex {
        StmtIndex {
            root: self.file.stmts.len(),
            patch: 0,
        }
    }

    fn this_stmt_index(&self)->StmtIndex {
        StmtIndex {
            root: self.file.stmts.len().saturating_sub(1),
            patch: 0,
        }
    }

    fn convert_expr(&mut self, expr: PExpr)->ExprIndex {
        match expr {
            PExpr::Operation{left,right,op}=>{
                let left = self.convert_expr(*left);
                let right = self.convert_expr(*right);
                self.file.add_expr(Expr::Operation{left, right, op})
            },
            PExpr::Field{left, name}=>{
                let left = self.convert_expr(*left);
                self.file.add_expr(Expr::Field{left, name})
            },
            PExpr::Group(list)=>{
                let new_list = list
                    .into_iter()
                    .map(|e|self.convert_expr(e))
                    .collect::<Vec<_>>();
                self.file.add_expr(Expr::Group(new_list))
            },
            PExpr::Var(name)=>self.file.add_expr(Expr::RawVar(name)),
            PExpr::Number(n)=>self.file.add_expr(Expr::Number(n)),
            PExpr::String(s)=>self.file.add_expr(Expr::String(s)),
            PExpr::Borrow(inner)=>{
                let inner = self.convert_expr(*inner);
                self.file.add_expr(Expr::Borrow(inner))
            },
            PExpr::Deref(inner)=>{
                let inner = self.convert_expr(*inner);
                self.file.add_expr(Expr::Deref(inner))
            },
            PExpr::None=>self.file.add_expr(Expr::None),
        }
    }

    fn convert_stmt(&mut self, scope: ScopeIndex, expr: PStmt)->StmtReturn {
        match expr {
            PStmt::FunctionDef{is_proc, name, pattern, block}=>StmtReturn {
                function: Some(RawFunction {
                    owning_scope: scope,
                    is_proc,
                    name,
                    pattern,
                    block,
                }),
                scopes: Vec::new(),
                var: None,
            },
            PStmt::VarDef{mutable, name, data}=>{
                let mut data_index = None;
                if let Some(data) = data {
                    data_index = Some(self.convert_expr(data));
                }

                let def = self.file.add_var(VarMetadata {
                    in_scope: scope,

                    definition: self.this_stmt_index(),
                    init: data_index,
                    disown: None,

                    data_type: Type::Undetermined,

                    borrows: Vec::new(),
                    uses: Vec::new(),
                    derefs: Vec::new(),
                    assigns: Vec::new(),

                    mem_loc: MemoryLocation::Undetermined,

                    mutable,
                    name,
                });

                self.file.add_stmt(Stmt::VarDef(def));

                StmtReturn {
                    var: Some((name, def)),
                    function: None,
                    scopes: Vec::new(),
                }
            },
            PStmt::VarSet{name, data}=>{
                let data = self.convert_expr(data);

                self.file.add_stmt(Stmt::VarSet{
                    name,
                    data,
                    var: VarIndex::invalid(),
                });

                StmtReturn {
                    function: None,
                    var: None,
                    scopes: Vec::new(),
                }
            },
            PStmt::IfElse{condition, block, default}=>{
                let mut scopes = Vec::new();

                let condition = self.convert_expr(condition);

                let block = self.convert_block(block);
                scopes.push(block.scope);

                let else_block = if let Some(else_block) = default {
                    let block = self.convert_block(else_block);
                    scopes.push(block.scope);
                    Some(block)
                } else {None};


                self.file.add_stmt(Stmt::IfElse {
                    condition,
                    block,
                    else_block,
                    last: self.this_stmt_index(),
                });

                StmtReturn {
                    function: None,
                    var: None,
                    scopes,
                }
            },
            PStmt::Conditional{conditions, actions}=>{
                let mut scopes = Vec::new();
                let conditions = conditions
                    .into_iter()
                    .map(|expr|self.convert_expr(expr))
                    .collect::<Vec<_>>();
                let actions = actions
                    .into_iter()
                    .map(|act|match act {
                        PCondAct::Expr(e)=>ConditionalAction::Expr(self.convert_expr(e)),
                        PCondAct::Scope(block)=>{
                            let block = self.convert_block(block);
                            scopes.push(block.scope);
                            ConditionalAction::Scope(block)
                        },
                    })
                    .collect::<Vec<_>>();

                self.file.add_stmt(Stmt::Conditional{
                    conditions,
                    actions,
                    last: self.this_stmt_index(),
                });

                StmtReturn {
                    function: None,
                    var: None,
                    scopes,
                }
            },
            PStmt::Scope(block)=>{
                let block = self.convert_block(block);

                StmtReturn {
                    function: None,
                    var: None,
                    scopes: vec![block.scope],
                }
            },
            PStmt::Disown(e)=>{
                let expr = self.convert_expr(e);

                self.file.add_stmt(Stmt::Disown(expr));

                StmtReturn {
                    function: None,
                    var: None,
                    scopes: Vec::new(),
                }
            },
            PStmt::Return(opt)=>{
                let mut expr = None;
                if let Some(e) = opt {
                    expr = Some(self.convert_expr(e));
                }

                self.file.add_stmt(Stmt::Return(expr));

                StmtReturn {
                    function: None,
                    var: None,
                    scopes: Vec::new(),
                }
            },
            PStmt::Expr(e)=>{
                let expr = self.convert_expr(e);

                self.file.add_stmt(Stmt::Expr(expr));

                StmtReturn {
                    function: None,
                    var: None,
                    scopes: Vec::new(),
                }
            },
        }
    }

    fn convert_block(&mut self, PBlock(stmts): PBlock)->Block {
        let scope_index = self.file.add_scope(Scope {
            first: self.next_stmt_index(),
            last: self.next_stmt_index(),
            functions: FnvHashMap::default(),
            scopes: Vec::new(),
            stack_slots: 0,
            vars: FnvHashMap::default(),
        });
        let first = self.next_stmt_index();

        let mut functions = Vec::new();

        for stmt in stmts {
            let mut ret = self.convert_stmt(scope_index, stmt);

            if let Some((name, index)) = ret.var {
                self.file.scopes[scope_index.0]
                    .vars
                    .entry(name)
                    .or_default()
                    .push(index);
            }

            if let Some(function) = ret.function {
                functions.push(function);
            }

            self.file.scopes[scope_index.0]
                .scopes
                .append(&mut ret.scopes);
        }

        let last = self.this_stmt_index();
        self.file.scopes[scope_index.0].last = last;

        let block = Block {
            first,
            last,
            scope: scope_index,
        };

        self.raw_func_queue.append(&mut functions);

        return block;
    }

    fn convert_function(&mut self, func: RawFunction) {
        let pattern = Rc::new(func.pattern);
        let block = self.convert_block(func.block);

        let index = self.file.add_function(FunctionDef {
            is_proc: func.is_proc,
            name: func.name,
            pattern: pattern.clone(),
            block,
        });

        self.file
            .get_mut_scope(func.owning_scope)
            .functions
            .entry(func.name)
            .or_default()
            .insert(pattern, index);
    }
}

struct StmtReturn {
    function: Option<RawFunction>,
    var: Option<(Name, VarIndex)>,
    scopes: Vec<ScopeIndex>,
}

struct RawFunction {
    pub owning_scope: ScopeIndex,
    pub is_proc: bool,
    pub name: Name,
    pub pattern: Pattern,
    pub block: PBlock,
}


#[inline]
pub fn convert_parse_tree(stmts: Vec<PStmt>)->File {
    FileConversion::convert(stmts)
}
