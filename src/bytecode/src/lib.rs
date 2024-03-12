pub use constant::Value;
pub use operator::{BinOp, UnOp};
use serde::{Deserialize, Serialize};

mod constant;
mod operator;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ByteCode {
    LDC(Value),
    BINOP(BinOp),
    UNOP(UnOp),
}

impl ByteCode {
    pub fn new_ldc(v: impl Into<Value>) -> Self {
        ByteCode::LDC(v.into())
    }

    pub fn new_binop(op: BinOp) -> Self {
        ByteCode::BINOP(op)
    }

    pub fn new_unop(op: UnOp) -> Self {
        ByteCode::UNOP(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let ldc_int = ByteCode::new_ldc(42);
        let serialized = bincode::serialize(&ldc_int).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(ldc_int, deserialized);

        let ldc_float = ByteCode::new_ldc(42.0);
        let serialized = bincode::serialize(&ldc_float).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(ldc_float, deserialized);
        assert_ne!(ldc_int, deserialized);

        let binop = ByteCode::new_binop(BinOp::Add);
        let serialized = bincode::serialize(&binop).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(binop, deserialized);

        let unop = ByteCode::new_unop(UnOp::Neg);
        let serialized = bincode::serialize(&unop).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(unop, deserialized);
    }
}
