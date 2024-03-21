use std::fmt::Display;

use parser::{BlockSeq, Decl, Expr};
use bytecode::ByteCode;

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

impl Compiler {
    pub fn new(program: BlockSeq) -> Compiler {
        Compiler {
            bytecode: vec![],
            program
        }
    }

    pub fn compile_expr(expr:&Expr) -> Result<ByteCode, CompileError> {
        match expr {
            Expr::Integer(val) => {
                Ok(ByteCode::ldc(*val))
            },
            _ => unimplemented!()
        }
    }

    fn compile_decl(decl: Decl) -> Result<ByteCode,CompileError> {
        let code = match decl {
            Decl::ExprStmt(expr) => {
                Compiler::compile_expr(&expr)
            },
            _ => unimplemented!()
            // Decl::LetStmt(stmt) => {
            //     Ok(ByteCode::DONE)

            // },
            // Decl::Block(blk) => {
            //     Ok(ByteCode::DONE)
            // }
        }?;

        Ok(code)
    }

    pub fn compile(self) -> Result<Vec<ByteCode>, CompileError>{
        // println!("Compile");
        let mut bytecode: Vec<ByteCode> = vec![];
        let decls = self.program.decls;

        for decl in decls {
            let code = Compiler::compile_decl(decl)?;
            bytecode.push(code);
        }

        // Handle expr
        if let Some(expr) = self.program.last_expr {
            let code = Compiler::compile_expr(expr.as_ref())?;
            bytecode.push(code);
        }

        bytecode.push(ByteCode::DONE);

        Ok(bytecode)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use bytecode::ByteCode;
    use bytecode::ByteCode::*;
    use bytecode::Value;
    use parser::Parser;

    use super::Compiler;

    fn test_compile(inp:&str) -> Vec<ByteCode>{
        let parser = Parser::new_from_string(inp);
        let parsed = parser.parse().expect("Should parse");
        let comp = Compiler::new(parsed);
        comp.compile().expect("Should compile")
    }

    #[test]
    fn test_compile_simple() {
        let res = test_compile("42;");
        assert_eq!(res, vec![ByteCode::ldc(42), DONE])
    }
}
