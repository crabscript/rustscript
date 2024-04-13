use serde::{Deserialize, Serialize};

use crate::{BinOp, FrameType, UnOp, Value};

/// A symbol is a string that represents a variable name.
pub type Symbol = String;

/// A thread ID is a unique identifier for a thread.
pub type ThreadID = i64;

/// The bytecode instructions that the VM can execute. See ignite::micro_code crate for more information
/// and implementation details.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ByteCode {
    /// Signal that the thread has finished executing.
    DONE,
    /// Assign the top of the operant stack to the given symbol in the current environment.
    ASSIGN(Symbol),
    /// Load the value of the given symbol onto the operant stack.
    LD(Symbol),
    /// Load a constant value onto the operant stack.
    LDC(Value),
    /// Pop the top of the operant stack.
    POP,
    /// Perform the given binary operation on the top two elements of the operant stack.
    BINOP(BinOp),
    /// Perform the given unary operation on the top of the operant stack.
    UNOP(UnOp),
    /// Jump to the given offset if the top of the operant stack is false.
    JOF(usize),
    /// Set pc to the given value.
    GOTO(usize),
    /// Keep popping the runtime stack until the given frame type is found.
    RESET(FrameType),
    /// Create a new scope with the given symbols.
    ENTERSCOPE(Vec<Symbol>),
    /// Exit the current scope.
    EXITSCOPE,
    /// Load the function with the given number of arguments and the function address onto the operant stack.
    LDF(usize, Vec<Symbol>),
    /// Call a function with the given number of arguments.
    CALL(usize),
    /// Spawn a new thread with the address of the instruction for the child to execute.
    SPAWN(usize),
    /// Join a thread.
    JOIN(ThreadID),
    /// Yield the current thread.
    YIELD,
}

/// For creating ByteCode instructions in a more ergonomic way.
impl ByteCode {
    pub fn ldc(v: impl Into<Value>) -> Self {
        ByteCode::LDC(v.into())
    }

    pub fn assign(sym: impl Into<Symbol>) -> Self {
        ByteCode::ASSIGN(sym.into())
    }

    pub fn ld(sym: impl Into<Symbol>) -> Self {
        ByteCode::LD(sym.into())
    }

    pub fn ldf<T: Into<Symbol>>(addr: usize, prms: Vec<T>) -> Self {
        ByteCode::LDF(addr, prms.into_iter().map(Into::into).collect())
    }

    pub fn binop(op: impl Into<BinOp>) -> Self {
        ByteCode::BINOP(op.into())
    }

    pub fn unop(op: impl Into<UnOp>) -> Self {
        ByteCode::UNOP(op.into())
    }

    pub fn reset(t: impl Into<FrameType>) -> Self {
        ByteCode::RESET(t.into())
    }

    pub fn enterscope<T: Into<Symbol>>(syms: Vec<T>) -> Self {
        ByteCode::ENTERSCOPE(syms.into_iter().map(Into::into).collect())
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
