use constant::{RawValue, Type};
use serde::{Deserialize, Serialize};

mod constant;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ByteCode {
    LDC(Type, RawValue),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let bytecode = ByteCode::LDC(Type::Int, 42);
        let serialized = bincode::serialize(&bytecode).unwrap();
        let deserialized: ByteCode = bincode::deserialize(&serialized).unwrap();
        assert_eq!(bytecode, deserialized);
    }
}
