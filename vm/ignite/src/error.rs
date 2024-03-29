use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("File is not a .o2 file: {0}")]
    NotO2File(String),

    #[error("symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("operand stack underflow")]
    OperandStackUnderflow,

    #[error("runtime stack underflow")]
    RuntimeStackUnderflow,

    #[error("pc out of bounds: {0}")]
    PcOutOfBounds(usize),

    #[error("bad type: expected {expected}, found {found}")]
    BadType { expected: String, found: String },

    #[error("bad argument: {0}")]
    IllegalArgument(String),

    #[error("arity and params mismatch: arity {arity}, found {params} params")]
    ArityParamsMismatch { arity: usize, params: usize },

    #[error("unimplemented")]
    Unimplemented,

    #[error("generic error: {0}")]
    Generic(String),
}
