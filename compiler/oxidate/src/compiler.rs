use anyhow::Result;
use std::fmt::Display;
use types::type_checker::TypeChecker;

use bytecode::{ByteCode, Value};
use parser::structs::{BinOpType, BlockSeq, Decl, Expr, IfElseData, UnOpType};

pub struct Compiler {
    program: BlockSeq,
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
        op: &UnOpType,
        expr: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        Compiler::compile_expr(expr, arr)?;
        match op {
            UnOpType::Negate => arr.push(ByteCode::UNOP(bytecode::UnOp::Neg)),
            UnOpType::Not => arr.push(ByteCode::UNOP(bytecode::UnOp::Not)),
        }
        Ok(())
    }

    // TODO: how to do type checking here?
    // Distinct phase before compilation is reached? Assign types to all expressions
    fn compile_binop(
        op: &BinOpType,
        lhs: &Expr,
        rhs: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        Compiler::compile_expr(lhs, arr)?;
        Compiler::compile_expr(rhs, arr)?;
        match op {
            BinOpType::Add => arr.push(ByteCode::BINOP(bytecode::BinOp::Add)),
            BinOpType::Mul => arr.push(ByteCode::BINOP(bytecode::BinOp::Mul)),
            BinOpType::Div => arr.push(ByteCode::BINOP(bytecode::BinOp::Div)),
            BinOpType::Sub => arr.push(ByteCode::BINOP(bytecode::BinOp::Sub)),
        }

        Ok(())
    }

    pub fn compile_expr(expr: &Expr, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Float(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Bool(val) => arr.push(ByteCode::ldc(*val)),
            Expr::BinOpExpr(op, lhs, rhs) => {
                Compiler::compile_binop(op, lhs, rhs, arr)?;
            }
            Expr::UnOpExpr(op, expr) => {
                Compiler::compile_unop(op, expr, arr)?;
            }
            // Load symbol
            Expr::Symbol(sym) => {
                arr.push(ByteCode::LD(sym.to_string()));
            }
            Expr::BlockExpr(blk) => {
                Compiler::compile_block(blk, arr)?;
            }
            Expr::IfElseExpr(if_else) => Compiler::compile_if_else(if_else, arr)?,
        }

        Ok(())
    }

    fn compile_assign(
        ident: &String,
        expr: &Expr,
        arr: &mut Vec<ByteCode>,
    ) -> Result<(), CompileError> {
        Compiler::compile_expr(expr, arr)?;

        let assign = ByteCode::ASSIGN(ident.to_owned());
        arr.push(assign);

        // Load unit after stmt to be consistent with popping after every stmt
        arr.push(ByteCode::LDC(Value::Unit));

        Ok(())
    }

    /// Compiles block body without checking if need to push Unit at the end.
    // So we can call this when compiling from global block to avoid pushing Unit there
    fn compile_block_body(blk: &BlockSeq, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        let decls = &blk.decls;
        let syms = &blk.symbols;

        if !syms.is_empty() {
            arr.push(ByteCode::ENTERSCOPE(syms.clone()));
        }

        for decl in decls {
            Compiler::compile_decl(decl, arr)?;
            // pop result of statements - need to ensure all stmts produce something (either Unit or something else)
            arr.push(ByteCode::POP);
        }

        // Handle expr
        if let Some(expr) = &blk.last_expr {
            Compiler::compile_expr(expr.as_ref(), arr)?;
        }

        if !syms.is_empty() {
            arr.push(ByteCode::EXITSCOPE);
        }

        Ok(())
    }

    /// Compile block appropriately based on whether it is none-like
    fn compile_block(blk: &BlockSeq, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        Compiler::compile_block_body(blk, arr)?;

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

    fn compile_decl(decl: &Decl, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        match decl {
            Decl::ExprStmt(expr) => {
                Compiler::compile_expr(expr, arr)?;
            }
            Decl::LetStmt(stmt) => {
                Compiler::compile_assign(&stmt.ident, &stmt.expr, arr)?;
            }
            Decl::AssignStmt(stmt) => {
                Compiler::compile_assign(&stmt.ident, &stmt.expr, arr)?;
            }
            Decl::IfOnlyStmt(if_else) => Compiler::compile_if_else(if_else, arr)?,
        };

        Ok(())
    }

    /// Compile if_else as statement or as expr - changes how blocks are compiled
    fn compile_if_else(if_else: &IfElseData, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        Compiler::compile_expr(&if_else.cond, arr)?;
        let jof_idx = arr.len();
        arr.push(ByteCode::JOF(0));

        Compiler::compile_block(&if_else.if_blk, arr)?;

        let goto_idx = arr.len();
        arr.push(ByteCode::GOTO(0));

        // set JOF arg to after GOTO (either else_blk start, or LDC Unit for if-only)
        let len = arr.len();
        if let Some(ByteCode::JOF(idx)) = arr.get_mut(jof_idx) {
            *idx = len;
        }

        if let Some(else_blk) = &if_else.else_blk {
            Compiler::compile_block(else_blk, arr)?;
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

    pub fn compile(self) -> anyhow::Result<Vec<ByteCode>, CompileError> {
        let mut bytecode: Vec<ByteCode> = vec![];
        Compiler::compile_block_body(&self.program, &mut bytecode)?;
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

#[cfg(test)]
mod tests {

    use std::vec;

    use bytecode::ByteCode;
    use bytecode::ByteCode::*;
    use bytecode::Value::*;
    use parser::Parser;

    use super::Compiler;

    fn exp_compile_str(inp: &str) -> Vec<ByteCode> {
        let parser = Parser::new_from_string(inp);
        let parsed = parser.parse().expect("Should parse");
        dbg!("parsed:", &parsed);
        let comp = Compiler::new(parsed);
        comp.compile().expect("Should compile")
    }

    fn test_comp(inp: &str, exp: Vec<ByteCode>) {
        let res = exp_compile_str(inp);
        dbg!(&res);
        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_simple() {
        let res = exp_compile_str("42;");
        assert_eq!(res, vec![ByteCode::ldc(42), POP, DONE]);

        let res = exp_compile_str("42; 45; 30");
        assert_eq!(
            res,
            vec![
                ByteCode::ldc(42),
                POP,
                ByteCode::ldc(45),
                POP,
                ByteCode::ldc(30),
                DONE
            ]
        );

        let res = exp_compile_str("42; true; 2.36;");
        assert_eq!(
            res,
            vec![
                ByteCode::ldc(42),
                POP,
                ByteCode::ldc(true),
                POP,
                ByteCode::ldc(2.36),
                POP,
                DONE
            ]
        )
    }

    #[test]
    fn test_compile_binop() {
        let res = exp_compile_str("2+3*2-4;");
        let exp = vec![
            LDC(Int(2)),
            LDC(Int(3)),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Mul),
            BINOP(bytecode::BinOp::Add),
            LDC(Int(4)),
            BINOP(bytecode::BinOp::Sub),
            POP,
            DONE,
        ];

        assert_eq!(res, exp);

        let res = exp_compile_str("2+3*4-5/5");

        let exp = [
            LDC(Int(2)),
            LDC(Int(3)),
            LDC(Int(4)),
            BINOP(bytecode::BinOp::Mul),
            BINOP(bytecode::BinOp::Add),
            LDC(Int(5)),
            LDC(Int(5)),
            BINOP(bytecode::BinOp::Div),
            BINOP(bytecode::BinOp::Sub),
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_let() {
        let res = exp_compile_str("let x = 2;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);

        // stmt last
        let res = exp_compile_str("let x = 2; let y = 3; ");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);

        // many
        let res = exp_compile_str("let x = 2; let y = 3; 40");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(40)),
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_sym() {
        let res = exp_compile_str("let x = 2; -x+2;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            UNOP(bytecode::UnOp::Neg),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Add),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);

        let res = exp_compile_str("let x = 2; let y = x; x*5+2");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string(), "y".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LD("x".to_string()),
            LDC(Int(5)),
            BINOP(bytecode::BinOp::Mul),
            LDC(Int(2)),
            BINOP(bytecode::BinOp::Add),
            EXITSCOPE,
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_not() {
        let res = exp_compile_str("!true");
        let exp = [LDC(Bool(true)), UNOP(bytecode::UnOp::Not), DONE];
        assert_eq!(res, exp);

        let res = exp_compile_str("!!false");
        let exp = [
            LDC(Bool(false)),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            DONE,
        ];
        assert_eq!(res, exp);

        let res = exp_compile_str("!!!true;");
        let exp = [
            LDC(Bool(true)),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            UNOP(bytecode::UnOp::Not),
            POP,
            DONE,
        ];
        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_assign() {
        let res = exp_compile_str("let x = 2; x = 3;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Int(3)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);

        // diff types
        let res = exp_compile_str("let x = 2; x = true;");
        let exp = vec![
            ENTERSCOPE(vec!["x".to_string()]),
            LDC(Int(2)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            LDC(Bool(true)),
            ASSIGN("x".to_string()),
            LDC(Unit),
            POP,
            EXITSCOPE,
            DONE,
        ];
        assert_eq!(res, exp);
    }

    use bytecode::Value::*;
    #[test]
    fn test_compile_blk_simple() {
        let t = "{ 2 }";
        let exp = vec![ByteCode::ldc(2), DONE];
        test_comp(t, exp);

        let t = "{ 2; 3 }";
        let exp = vec![ByteCode::ldc(2), ByteCode::POP, ByteCode::ldc(3), DONE];
        test_comp(t, exp);

        let t = "{ 2; 3; }";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            LDC(Unit),
            DONE,
        ];
        test_comp(t, exp);

        let t = "{ 2; 3; 4 }";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            DONE,
        ];
        test_comp(t, exp);

        // // like doing just 4;
        let t = "{ 2; 3; 4 };";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            ByteCode::POP,
            DONE,
        ];
        test_comp(t, exp);

        let t = "{ 2; 3; 4; };";
        let exp = vec![
            ByteCode::ldc(2),
            ByteCode::POP,
            ByteCode::ldc(3),
            ByteCode::POP,
            ByteCode::ldc(4),
            ByteCode::POP,
            ByteCode::ldc(Unit),
            ByteCode::POP,
            DONE,
        ];
        test_comp(t, exp);
    }

    #[test]
    fn test_compile_blk_cases() {
        test_comp("{ 2 }", vec![ByteCode::ldc(2), DONE]);
        // blk with no last expr or none_like returns Unit
        test_comp("{ 2; }", vec![ByteCode::ldc(2), POP, LDC(Unit), DONE]);

        // // since we pop after every stmt, if the block ends in expr we just rely on that
        test_comp("{ 2 };", vec![ByteCode::ldc(2), POP, DONE]);

        // // we pop after every stmt, but since this blk has no last expr we push unit before blk ends so the pop doesn't
        test_comp(
            "{ 2; };",
            vec![ByteCode::ldc(2), POP, ByteCode::ldc(Unit), POP, DONE],
        );

        // nested
        test_comp(
            r"
        {
            2;
            {
                {

                }
            }
        }
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), DONE],
        );

        // nested
        test_comp(
            r"
        {
            2;
            {
                {

                }
            }
        };
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), POP, DONE],
        );

        // nested with stmt inside
        test_comp(
            r"
        {
            2;
            {
                { 
                    {

                    };
                }
            }
        }
        ",
            vec![LDC(Int(2)), POP, LDC(Unit), POP, LDC(Unit), DONE],
        );
    }

    #[test]
    fn test_compile_blk_let() {
        // empty blk
        let t = r"
        let x = {
            {}
        };
        ";

        // last LDC Unit if from compiling let. last POP is from automatic pop after decl
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                LDC(Unit),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );

        let t = r"
        let x = 2;
        {
            let y = 3;
            x+y
        }
        ";
        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                ByteCode::ldc(Unit),
                POP,
                ENTERSCOPE(vec!["y".to_string()]),
                LDC(Int(3)),
                ASSIGN("y".to_string()),
                LDC(Unit),
                POP,
                LD("x".to_string()),
                LD("y".to_string()),
                ByteCode::binop("+"),
                EXITSCOPE,
                EXITSCOPE,
                DONE,
            ],
        );

        let t = r"
        let x = 2; { {2+2;} };
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                POP,
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );

        // nested none-like
        let t = r"
        let x = 2; { 

            {
                {
                    2+2;
                }
            } 
        
        };
        ";

        test_comp(
            t,
            vec![
                ENTERSCOPE(vec!["x".to_string()]),
                ByteCode::ldc(2),
                ASSIGN("x".to_string()),
                LDC(Unit),
                POP,
                LDC(Int(2)),
                LDC(Int(2)),
                ByteCode::binop("+"),
                POP,
                LDC(Unit),
                POP,
                EXITSCOPE,
                DONE,
            ],
        );
    }

    #[test]
    fn test_compile_if_only() {
        // if only with nothing after
        let t = r"
        if !true {
            2
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                ByteCode::unop("!"),
                JOF(5),
                LDC(Int(2)),
                GOTO(6),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // ifonly-blk has value
        let t = r"
        if !true {
            2
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                ByteCode::unop("!"),
                JOF(5),
                LDC(Int(2)),
                GOTO(6),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // if only-blk none like
        let t = r"
        if true {
            2;
            3;
        }
        200
        ";

        test_comp(
            t,
            vec![
                LDC(Bool(true)),
                JOF(8),
                LDC(Int(2)),
                POP,
                LDC(Int(3)),
                POP,
                LDC(Unit),
                GOTO(9),
                LDC(Unit),
                POP,
                LDC(Int(200)),
                DONE,
            ],
        );

        // consec
        let t = r"
        let y = true;
        if false {
           2; 3 
        }

        if y {  
            y = false;
        }

        y
        ";

        let exp = vec![
            ENTERSCOPE(vec!["y".to_string()]),
            LDC(Bool(true)),
            ByteCode::ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Bool(false)),
            JOF(11),
            LDC(Int(2)),
            POP,
            LDC(Int(3)),
            GOTO(12),
            LDC(Unit),
            POP,
            ByteCode::ld("y"),
            JOF(21),
            LDC(Bool(false)),
            ByteCode::ASSIGN("y".to_string()),
            LDC(Unit),
            POP,
            LDC(Unit),
            GOTO(22),
            LDC(Unit),
            POP,
            ByteCode::ld("y"),
            EXITSCOPE,
            DONE,
        ];

        test_comp(t, exp);
    }
}
