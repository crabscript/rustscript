use std::{error::Error, fmt::Display};
use anyhow::Result;

use parser::{BinOpType, BlockSeq, Decl, Expr, Parser};
use bytecode::{ByteCode, Value};

pub struct Compiler {
    bytecode: Vec<ByteCode>,
    program: BlockSeq
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
        write!(f, "[CompileError]: {}", self.msg)
    }
}

impl std::error::Error for CompileError {}

impl Compiler {
    pub fn new(program: BlockSeq) -> Compiler {
        Compiler {
            bytecode: vec![],
            program
        }
    }
    
    // TODO: how to do type checking here?
        // Distinct phase before compilation is reached? Assign types to all expressions
    fn compile_binop(op: &BinOpType, lhs: &Box<Expr>, rhs: &Box<Expr>, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        Compiler::compile_expr(lhs.as_ref(), arr)?;
        Compiler::compile_expr(rhs.as_ref(), arr)?;
        match op {
            BinOpType::Add => arr.push(ByteCode::BINOP(bytecode::BinOp::Add)),
            BinOpType::Mul => arr.push(ByteCode::BINOP(bytecode::BinOp::Mul)),
            BinOpType::Div => arr.push(ByteCode::BINOP(bytecode::BinOp::Div)),
            BinOpType::Sub => arr.push(ByteCode::BINOP(bytecode::BinOp::Sub)),
            _ => unimplemented!()
        }

        Ok(())
    }

    pub fn compile_expr(expr:&Expr, arr: &mut Vec<ByteCode>) -> Result<(), CompileError> {
        match expr {
            Expr::Integer(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Float(val) => arr.push(ByteCode::ldc(*val)),
            Expr::Bool(val) => arr.push(ByteCode::ldc(*val)),
            Expr::BinOpExpr(op, lhs, rhs) => {
                Compiler::compile_binop(op, lhs, rhs, arr)?;
            },
            _ => unimplemented!()
        }

        Ok(())
    }

    fn compile_decl(decl: Decl, arr: &mut Vec<ByteCode>) -> Result<(),CompileError> {
        match decl {
            Decl::ExprStmt(expr) => {
                Compiler::compile_expr(&expr, arr)?;
                // arr.push(code);
            },
            Decl::LetStmt(stmt) => {
                let ident = stmt.ident.to_string();
                let expr = stmt.expr;

                Compiler::compile_expr(&expr, arr)?;
                // arr.push(compiled_expr);

                let assign = ByteCode::ASSIGN(ident);
                arr.push(assign);

                // Load unit after stmt to be consistent with popping after every stmt
                arr.push(ByteCode::LDC(Value::Unit));
            },
            _ => unimplemented!()
            // Decl::LetStmt(stmt) => {
            //     Ok(ByteCode::DONE)

            // },
            // Decl::Block(blk) => {
            //     Ok(ByteCode::DONE)
            // }
        };

        Ok(())
    }

    pub fn compile(self) -> anyhow::Result<Vec<ByteCode>, CompileError>{
        // println!("Compile");
        let mut bytecode: Vec<ByteCode> = vec![];
        let decls = self.program.decls;

        for decl in decls {
            Compiler::compile_decl(decl, &mut bytecode)?;
            // pop result of statements - need to ensure all stmts produce something (either Unit or something else)
            bytecode.push(ByteCode::POP); 
        }

        // Handle expr
        if let Some(expr) = self.program.last_expr {
            Compiler::compile_expr(expr.as_ref(), &mut bytecode)?;
            // bytecode.push(code);
        }

        bytecode.push(ByteCode::DONE);

        Ok(bytecode)
    }
}

#[cfg(test)]
mod tests {

    use bytecode::ByteCode;
    use bytecode::ByteCode::*;
    use parser::Parser;
    use bytecode::Value::*;

    use super::Compiler;

    fn exp_compile_str(inp:&str) -> Vec<ByteCode>{
        let parser = Parser::new_from_string(inp);
        let parsed = parser.parse().expect("Should parse");
        let comp = Compiler::new(parsed);
        comp.compile().expect("Should compile")
    }

    #[test]
    fn test_compile_simple() {
        let res = exp_compile_str("42;");
        assert_eq!(res, vec![ByteCode::ldc(42), POP, DONE]);

        let res = exp_compile_str("42; 45; 30");
        assert_eq!(res, vec![ByteCode::ldc(42), POP, ByteCode::ldc(45), POP, ByteCode::ldc(30), DONE]);

        let res = exp_compile_str("42; true; 2.36;");
        assert_eq!(res, vec![ByteCode::ldc(42), POP, ByteCode::ldc(true), POP, ByteCode::ldc(2.36), POP, DONE])
    }

    #[test]
    fn test_compile_binop() {
        let res = exp_compile_str("2+3*2-4;");
        let exp = vec![
            LDC(
                Int(
                    2,
                ),
            ),
            LDC(
                Int(
                    3,
                ),
            ),
            LDC(
                Int(
                    2,
                ),
            ),
            BINOP(
                bytecode::BinOp::Mul,
            ),
            BINOP(
                bytecode::BinOp::Add,
            ),
            LDC(
                Int(
                    4,
                ),
            ),
            BINOP(
                bytecode::BinOp::Sub,
            ),
            POP,
            DONE,
        ];
        
        assert_eq!(res, exp);

        let res = exp_compile_str("2+3*4-5/5");

        let exp = [
            LDC(
                Int(
                    2,
                ),
            ),
            LDC(
                Int(
                    3,
                ),
            ),
            LDC(
                Int(
                    4,
                ),
            ),
            BINOP(
                bytecode::BinOp::Mul,
            ),
            BINOP(
                bytecode::BinOp::Add,
            ),
            LDC(
                Int(
                    5,
                ),
            ),
            LDC(
                Int(
                    5,
                ),
            ),
            BINOP(
                bytecode::BinOp::Div,
            ),
            BINOP(
                bytecode::BinOp::Sub,
            ),
            DONE,
        ];

        assert_eq!(res, exp);
    }

    #[test]
    fn test_compile_let() {
        let res = exp_compile_str("let x = 2;");
        let exp = vec![
            LDC(
                Int(
                    2,
                ),
            ),
            ASSIGN(
                "x".to_string(),
            ),
            LDC(
                Unit,
            ),
            POP,
            DONE,
        ];
        
        assert_eq!(res, exp);

        // stmt last
        let res = exp_compile_str("let x = 2; let y = 3; ");
        let exp = vec![
            LDC(
                Int(
                    2,
                ),
            ),
            ASSIGN(
                "x".to_string(),
            ),
            LDC(
                Unit,
            ),
            POP,
            LDC(
                Int(
                    3,
                ),
            ),
            ASSIGN(
                "y".to_string(),
            ),
            LDC(
                Unit,
            ),
            POP,
            DONE
        ];
        
        assert_eq!(res, exp);

        // many
        let res = exp_compile_str("let x = 2; let y = 3; 40");
        let exp = vec![
            LDC(
                Int(
                    2,
                ),
            ),
            ASSIGN(
                "x".to_string(),
            ),
            LDC(
                Unit,
            ),
            POP,
            LDC(
                Int(
                    3,
                ),
            ),
            ASSIGN(
                "y".to_string(),
            ),
            LDC(
                Unit,
            ),
            POP,
            LDC(
                Int(
                    40,
                ),
            ),
            DONE
        ];
        
        assert_eq!(res, exp);
    }
}
