use anyhow::Result;
use std::{fmt::Display, rc::Rc, vec};
use types::type_checker::TypeChecker;

use bytecode::{BinOp, ByteCode, Value};
use parser::structs::{
    BinOpType, BlockSeq, Decl, Expr, FnCallData, FnDeclData, IfElseData, LoopData, UnOpType,
};

pub struct Compiler {
    program: BlockSeq,
    // Tracks idx in bytecode for any nested break stmts compiled for that loop. Stack of vecs since we can have nested loops
    // and break should only break the closest enclosing loop
    loop_stack: Vec<Vec<usize>>,
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

// Workaround to ensure builtins that dont pop produce Unit when compiling fn call
// Because user functions even if empty will produce unit (everything is value producing), so
// this issue only applies to builtins with no value pushed
const BUILTINS_WITH_NO_VAL: [&str; 3] = ["println", "print", "sem_set"];

impl Compiler {
    pub fn new(program: BlockSeq) -> Compiler {
        Compiler {
            program,
            loop_stack: vec![],
        }
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
            Expr::FnCallExpr(fn_call) => self.compile_fn_call(fn_call, arr)?,
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
            Decl::LoopStmt(lp) => self.compile_loop(lp, arr)?,
            // push GOTO, push idx of this break in arr onto loop stack
            Decl::BreakStmt => {
                let break_idx = arr.len();
                arr.push(ByteCode::GOTO(0));
                if let Some(breaks) = self.loop_stack.last_mut() {
                    breaks.push(break_idx);
                }
            }
            Decl::FnDeclStmt(fn_decl) => self.compile_fn_decl(fn_decl, arr)?,
            Decl::ReturnStmt(ret_stmt) => {
                // compile expr. if not there, push Unit
                if let Some(expr) = ret_stmt {
                    self.compile_expr(expr, arr)?;
                } else {
                    arr.push(ByteCode::ldc(Value::Unit));
                }

                // push RESET
                arr.push(ByteCode::RESET(bytecode::FrameType::CallFrame))
            }
        };

        Ok(())
    }

    fn compile_fn_decl(
        &mut self,
        fn_decl: &FnDeclData,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        dbg!("GOT to compile fn decl: ", fn_decl);

        // we are about to push LDF and GOTO before fn compile
        let fn_start_idx = arr.len() + 2;

        let param_strs: Vec<String> = fn_decl.params.iter().map(|x| x.name.to_string()).collect();

        arr.push(ByteCode::ldf(fn_start_idx, param_strs));

        // push GOTO for skipping fn compile
        let goto_idx = arr.len();
        arr.push(ByteCode::GOTO(0));

        // add params to fn blk
        // let mut fn_blk = fn_decl.body.clone();
        // let mut param_names = fn_decl.params.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
        // fn_blk.symbols.append(&mut param_names);

        // compile the augmented blk

        self.compile_block(&fn_decl.body, arr)?;
        // self.compile_block(&fn_blk, arr)?;

        // push reset to return last value produced by blk, in case no return was there
        arr.push(ByteCode::RESET(bytecode::FrameType::CallFrame));

        // GOTO will jump to ASSIGN, ASSIGN pops closure and then we load Unit so no underflow
        let goto_addr = arr.len();
        arr.push(ByteCode::assign(&fn_decl.name));
        arr.push(ByteCode::ldc(Value::Unit));

        // patch GOTO
        if let Some(ByteCode::GOTO(idx)) = arr.get_mut(goto_idx) {
            *idx = goto_addr;
        }

        Ok(())
    }

    /// Function call expression e.g println(2,3)
    fn compile_fn_call(
        &mut self,
        fn_call: &FnCallData,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        // TODO: change to accept arbitary expr for fn
        self.compile_expr(&Expr::Symbol(fn_call.name.clone()), arr)?;

        for arg in fn_call.args.iter() {
            self.compile_expr(arg, arr)?;
        }

        arr.push(ByteCode::CALL(fn_call.args.len()));

        // push unit for builtin that produces no value
        if BUILTINS_WITH_NO_VAL.contains(&fn_call.name.as_str()) {
            arr.push(ByteCode::ldc(Value::Unit));
        }

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

    /*Assumptions:
    1. Before entering a statement, op_stack length  is 0
    2. Upon jump on false, op stack length is 0
    */
    // Returns index in pc of LDC unit for the loop
    fn compile_loop_inner(
        &mut self,
        loop_data: &LoopData,
        arr: &mut Vec<ByteCode>,
    ) -> Result<usize, CompileError> {
        // dbg!("compile loop, stack:", &self.loop_stack);
        let loop_start = arr.len();
        // only need to patch JOF if condition was present

        let mut jof_idx: Option<usize> = None;
        if let Some(expr) = &loop_data.cond {
            self.compile_expr(expr, arr)?;
            jof_idx.replace(arr.len());
            arr.push(ByteCode::JOF(0));
        }

        // loop body
        self.compile_block(&loop_data.body, arr)?;
        arr.push(ByteCode::POP); // pop value produced by blk
        arr.push(ByteCode::GOTO(loop_start)); // goto start of loop

        let loop_end_idx = arr.len(); // JOF and break must jump to LDC Unit
        arr.push(ByteCode::LDC(Value::Unit)); // loop produces Unit (popped by decl loop since stmt)

        // patch JOF
        if let Some(idx) = jof_idx {
            if let Some(ByteCode::JOF(jmp_idx)) = arr.get_mut(idx) {
                *jmp_idx = loop_end_idx;
            }
        }

        Ok(loop_end_idx)
    }

    // To ensure loop stack is always popped / pushed whether err or not - like calling defer in Go
    fn compile_loop(
        &mut self,
        loop_data: &LoopData,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        self.loop_stack.push(vec![]);
        let end_idx = self.compile_loop_inner(loop_data, arr);

        let end_idx = end_idx?;

        // patch all the break stmts
        let breaks = self
            .loop_stack
            .last()
            .expect("Loop stack should be present since pushed earlier");

        // Later: can use this to detect infinite loops
        // if breaks.len() == 0 && loop_data.cond.is_none() {
        //     dbg!("[WARNING] Breaks was empty: loop has no break");
        // }

        for idx in breaks.iter() {
            let idx = idx.to_owned();

            if let Some(ByteCode::GOTO(break_idx)) = arr.get_mut(idx) {
                *break_idx = end_idx;
            }
        }

        self.loop_stack.pop();
        Ok(())
    }

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
