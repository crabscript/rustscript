use std::collections::HashMap;
use std::fmt::Display;

use parser::{BlockSeq, Decl};

#[derive(Debug, PartialEq)]
pub struct TypeErrors {
    errs: Vec<String>,
}

impl TypeErrors {
    pub fn new() -> TypeErrors {
        TypeErrors { errs: vec![] }
    }

    pub fn add(&mut self, err: &str) {
        self.errs.push(err.to_string());
    }
}

impl Display for TypeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .errs
            .iter()
            .map(|x| format!("[TypeError]: {}", x))
            .collect::<Vec<String>>()
            .join("\n");
        write!(f, "{}", string)
    }
}

impl std::error::Error for TypeErrors {}

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

    /// Type check declaration and add errors if any
    fn check_decl(decl: &Decl, _errs: &mut TypeErrors) {
        dbg!(format!("Type check: {}", decl.to_string()));
    }

    pub fn type_check(self) -> Result<(), TypeErrors> {
        dbg!(&self.program);
        dbg!(&self.ty_env.len());

        let mut errs = TypeErrors::new();
        errs.add("Error 1");
        errs.add("Error 2");
        println!("{}", errs);

        for decl in self.program.decls.iter() {
            TypeChecker::check_decl(decl, &mut errs);
        }

        Ok(())
    }
}

impl Default for TypeErrors {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use parser::BlockSeq;
    use parser::Parser;

    use super::TypeChecker;

    fn expect_parse(inp: &str) -> BlockSeq {
        Parser::new_from_string(inp).parse().expect("Should parse")
    }

    #[test]
    fn test_type_check() {
        let prog = expect_parse("let x : int = 2; let y : bool = true;");
        let tyc = TypeChecker::new(prog).type_check();
        dbg!(tyc.is_err());
    }
}
