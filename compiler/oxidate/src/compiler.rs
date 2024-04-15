use anyhow::Result;
use std::{fmt::Display, rc::Rc, vec};
use types::type_checker::TypeChecker;

use bytecode::{BinOp, ByteCode, Value};
use parser::structs::{BinOpType, BlockSeq, Decl, Expr, IfElseData, UnOpType};

pub struct Compiler {
    program: BlockSeq,
    // Tracks idx in bytecode for any nested break stmts compiled for that loop. Stack of vecs since we can have nested loops
    // and break should only break the closest enclosing loop
    // loop_stack: Vec<Vec<usize>>
}

#[derive(Debug, PartialEq)]
pub struct CompileError {
    msg: String,
}

impl CompileError {
    pub fn new(err: &str) -> CompileError {
        CompileError {
            msg: err.to_owned(),
        }
    }
}

impl Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[CompileError] -  {}", self.msg)
    }
}

impl std::error::Error for CompileError {}

impl Compiler {
    pub fn new(program: BlockSeq) -> Compiler {
        Compiler { program }
    }

    fn compile_unop(
        &mut self,
        op: &UnOpType,
        expr: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        self.compile_expr(expr, arr)?;
        match op {
            UnOpType::Negate => arr.push(ByteCode::UNOP(bytecode::UnOp::Neg)),
            UnOpType::Not => arr.push(ByteCode::UNOP(bytecode::UnOp::Not)),
        }
        Ok(())
    }

    // And, Or - short-circuiting
    fn compile_and_or(
        &mut self,
        op: &BinOpType,
        lhs: &Expr,
        rhs: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        match op {
            // x && y => if x { y } else { false }
            // if true, keep going. else, return false out and stop
            BinOpType::LogicalAnd => {
                let if_blk = BlockSeq {
                    decls: vec![],
                    last_expr: Some(Rc::new(rhs.clone())),
                    symbols: vec![],
                };

                let else_blk = BlockSeq {
                    decls: vec![],
                    last_expr: Some(Rc::new(Expr::Bool(false))),
                    symbols: vec![],
                };

                let stmt = IfElseData {
                    cond: lhs.clone(),
                    if_blk,
                    else_blk: Some(else_blk),
                };

                self.compile_if_else(&stmt, arr)?;
            }
            // x || y => if x { true } else { y }
            // if x true, stop and return true. else, keep going
            BinOpType::LogicalOr => {
                let if_blk = BlockSeq {
                    decls: vec![],
                    last_expr: Some(Rc::new(Expr::Bool(true))),
                    symbols: vec![],
                };

                let else_blk = BlockSeq {
                    decls: vec![],
                    last_expr: Some(Rc::new(rhs.clone())),
                    symbols: vec![],
                };

                let stmt = IfElseData {
                    cond: lhs.clone(),
                    if_blk,
                    else_blk: Some(else_blk),
                };

                self.compile_if_else(&stmt, arr)?;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    // Distinct phase before compilation is reached? Assign types to all expressions
    fn compile_binop(
        &mut self,
        op: &BinOpType,
        lhs: &Expr,
        rhs: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        // avoid compiling exprs first for these
        if matches!(op, BinOpType::LogicalAnd | BinOpType::LogicalOr) {
            return self.compile_and_or(op, lhs, rhs, arr);
        }

        self.compile_expr(lhs, arr)?;
        self.compile_expr(rhs, arr)?;

        match op {
            BinOpType::Add => arr.push(ByteCode::BINOP(bytecode::BinOp::Add)),
            BinOpType::Mul => arr.push(ByteCode::BINOP(bytecode::BinOp::Mul)),
            BinOpType::Div => arr.push(ByteCode::BINOP(bytecode::BinOp::Div)),
            BinOpType::Sub => arr.push(ByteCode::BINOP(bytecode::BinOp::Sub)),
            BinOpType::Gt => arr.push(ByteCode::BINOP(BinOp::Gt)),
            BinOpType::Lt => arr.push(ByteCode::BINOP(BinOp::Lt)),
            BinOpType::LogicalEq => arr.push(ByteCode::BINOP(BinOp::Eq)),
            // Rest are and/or: handled above
            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn compile_expr(
        &mut self,
        expr: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Float(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Bool(val) => arr.push(ByteCode::ldc(*val)),
            Expr::BinOpExpr(op, lhs, rhs) => {
                self.compile_binop(op, lhs, rhs, arr)?;
            }
            Expr::UnOpExpr(op, expr) => {
                self.compile_unop(op, expr, arr)?;
            }
            // Load symbol
            Expr::Symbol(sym) => {
                arr.push(ByteCode::LD(sym.to_string()));
            }
            Expr::BlockExpr(blk) => {
                self.compile_block(blk, arr)?;
            }
            Expr::IfElseExpr(if_else) => self.compile_if_else(if_else, arr)?,
        }

        Ok(())
    }

    fn compile_assign(
        &mut self,
        ident: &String,
        expr: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        self.compile_expr(expr, arr)?;

        let assign = ByteCode::ASSIGN(ident.to_owned());
        arr.push(assign);

        // Load unit after stmt to be consistent with popping after every stmt
        arr.push(ByteCode::LDC(Value::Unit));

        Ok(())
    }

    /// Compiles block body without checking if need to push Unit at the end.
    // So we can call this when compiling from global block to avoid pushing Unit there
    fn compile_block_body(
        &mut self,
        blk: &BlockSeq,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        let decls = &blk.decls;
        let syms = &blk.symbols;

        if !syms.is_empty() {
            arr.push(ByteCode::ENTERSCOPE(syms.clone()));
        }

        for decl in decls {
            self.compile_decl(decl, arr)?;
            // pop result of statements - need to ensure all stmts produce something (either Unit or something else)
            arr.push(ByteCode::POP);
        }

        // Handle expr
        if let Some(expr) = &blk.last_expr {
            self.compile_expr(expr.as_ref(), arr)?;
        }

        if !syms.is_empty() {
            arr.push(ByteCode::EXITSCOPE);
        }

        Ok(())
    }

    /// Compile block appropriately based on whether it is none-like
    fn compile_block(
        &mut self,
        blk: &BlockSeq,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        self.compile_block_body(blk, arr)?;

        // does not produce value: return Unit
        if Compiler::blk_produces_nothing(blk) {
            arr.push(ByteCode::ldc(Value::Unit));
        }

        Ok(())
    }

    // blk is_none_like if it has no last expr: then we must push Unit as its last value
    // recursive check not needed as empty blks / blk without last also produce Unit
    fn blk_produces_nothing(blk: &BlockSeq) -> bool {
        blk.last_expr.is_none()
    }

    fn compile_decl(&mut self, decl: &Decl, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        match decl {
            Decl::ExprStmt(expr) => {
                self.compile_expr(expr, arr)?;
            }
            Decl::LetStmt(stmt) => {
                self.compile_assign(&stmt.ident, &stmt.expr, arr)?;
            }
            Decl::AssignStmt(stmt) => {
                self.compile_assign(&stmt.ident, &stmt.expr, arr)?;
            }
            Decl::IfOnlyStmt(if_else) => self.compile_if_else(if_else, arr)?,
            Decl::LoopStmt(_) => todo!(),
            Decl::BreakStmt => todo!(),
        };

        Ok(())
    }

    /// Compile if_else as statement or as expr - changes how blocks are compiled
    fn compile_if_else(
        &mut self,
        if_else: &IfElseData,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        self.compile_expr(&if_else.cond, arr)?;
        let jof_idx = arr.len();
        arr.push(ByteCode::JOF(0));

        self.compile_block(&if_else.if_blk, arr)?;

        let goto_idx = arr.len();
        arr.push(ByteCode::GOTO(0));

        // set JOF arg to after GOTO (either else_blk start, or LDC Unit for if-only)
        let len = arr.len();
        if let Some(ByteCode::JOF(idx)) = arr.get_mut(jof_idx) {
            *idx = len;
        }

        if let Some(else_blk) = &if_else.else_blk {
            self.compile_block(else_blk, arr)?;
        } else {
            // no else: push Unit so decl pop doesn't underflow if branch didn't run
            arr.push(ByteCode::ldc(Value::Unit));
        }

        // GOTO after the else / after load unit once if is done executing (when cond is true)
        let len = arr.len();
        if let Some(ByteCode::GOTO(idx)) = arr.get_mut(goto_idx) {
            *idx = len;
        }

        Ok(())
    }

    // fn compile_loop_inner(loop_data: &LoopData, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
    //     Ok(())
    // }

    // fn compile_loop(loop_data: &LoopData, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {

    //     Ok(())
    // }

    pub fn compile(mut self) -> anyhow::Result<Vec<ByteCode>, CompileError> {
        let mut bytecode: Vec<ByteCode> = vec![];
        let prog = self.program.clone();
        self.compile_block_body(&prog, &mut bytecode)?;
        bytecode.push(ByteCode::DONE);

        Ok(bytecode)
    }
}

/// Takes in a string and returns compiled bytecode or errors
pub fn compile_from_string(inp: &str, type_check: bool) -> Result<Vec<ByteCode>> {
    let parser = parser::Parser::new_from_string(inp);
    let program = parser.parse()?;

    if type_check {
        TypeChecker::new(&program).type_check()?;
    }

    let compiler = Compiler::new(program);
    Ok(compiler.compile()?)
}
