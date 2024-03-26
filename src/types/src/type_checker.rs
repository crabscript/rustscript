use std::collections::HashMap;

use parser::BlockSeq;

pub fn type_check() {
    println!("type checker");
}

/// Compile type
pub enum Type {
    Int,
    Float,
    Bool,
}

/// Struct to enable type checking by encapsulating type environment.
pub struct TypeChecker {
    program: BlockSeq,
    ty_env: HashMap<String, Type>,
}

impl TypeChecker {
    pub fn new(program: BlockSeq) -> TypeChecker {
        TypeChecker {
            program,
            ty_env: HashMap::new(),
        }
    }
    pub fn type_check(self) {
        dbg!(&self.program);
        dbg!(&self.ty_env.len());
    }
}
