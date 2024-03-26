pub mod compiler;

use anyhow::Result;
use bytecode::ByteCode;
use compiler::Compiler;
use parser::Parser;
// use types::type_checker::TypeChecker;

// Compiles a input string and returns bytecode array
pub fn compile_string(inp: &str) -> Result<Vec<ByteCode>> {
    let parser = Parser::new_from_string(inp);
    let parsed = parser.parse()?;

    // Check types
    // TypeChecker::new(&parsed).type_check()?;

    let comp = Compiler::new(parsed);

    let res = comp.compile()?;
    Ok(res)
}
