pub use constant::Value;
pub use io::*;
pub use operator::{BinOp, UnOp};
use serde::{Deserialize, Serialize};
pub use stack_frame::FrameType;

mod constant;
mod error;
mod io;
mod operator;
mod stack_frame;

/// A symbol is a string that represents a variable name.
pub type Symbol = String;

/// The bytecode instructions that the VM can execute.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ByteCode {
    DONE,
    ASSIGN(Symbol),
    LD(Symbol),
    LDC(Value),
    POP,
    BINOP(BinOp),
    UNOP(UnOp),
    JOF(usize),
    GOTO(usize),
    RESET(FrameType),
    ENTERSCOPE(Vec<Symbol>),
    EXITSCOPE,
}

impl ByteCode {
    pub fn ldc(v: impl Into<Value>) -> Self {
        ByteCode::LDC(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let ldc_int = ByteCode::ldc(42);
        let serialized = bincode::serialize(&ldc_int).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(ldc_int, deserialized);

        let ldc_float = ByteCode::ldc(42.0);
        let serialized = bincode::serialize(&ldc_float).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(ldc_float, deserialized);
        assert_ne!(ldc_int, deserialized);

        let binop = ByteCode::BINOP(BinOp::Add);
        let serialized = bincode::serialize(&binop).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(binop, deserialized);

        let unop = ByteCode::UNOP(UnOp::Neg);
        let serialized = bincode::serialize(&unop).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(unop, deserialized);
    }
}
