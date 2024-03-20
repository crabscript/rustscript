use std::fmt::Display;

use parser::{BlockSeq, Decl};
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

    fn compile_decl(decl: Decl) -> Result<ByteCode,CompileError> {

        Ok(ByteCode::DONE)
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

        bytecode.push(ByteCode::DONE);

        Ok(bytecode)
    }
}
