use std::vec;

use parser::{BlockSeq, Parser};
use bytecode::ByteCode;

pub struct Compiler {
    bytecode: Vec<ByteCode>,
    program: BlockSeq
}

impl Compiler {
    pub fn new(program: BlockSeq) -> Compiler {
        Compiler {
            bytecode: vec![],
            program
        }
    }

    pub fn compile(mut self) {
        println!("Compile");
        
    }
}
