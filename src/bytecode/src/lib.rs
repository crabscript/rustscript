use constant::Value;
use serde::{Deserialize, Serialize};

mod constant;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ByteCode {
    LDC(Value),
}

impl ByteCode {
    pub fn new_ldc(v: impl Into<Value>) -> Self {
        ByteCode::LDC(v.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let bc_int = ByteCode::new_ldc(42);
        let serialized = bincode::serialize(&bc_int).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(bc_int, deserialized);
        
        let bc_float = ByteCode::new_ldc(42.0);
        let serialized = bincode::serialize(&bc_float).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(bc_float, deserialized);
        assert_ne!(bc_int, deserialized);
    }
}
