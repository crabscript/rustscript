use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File does not exist: {0}")]
    FileDoesNotExist(String),

    #[error("File is not a .o2 file: {0}")]
    NotO2File(String),

    #[error("Unbounded name: {0}")]
    UnboundedName(String),

    #[error("Operand stack underflow")]
    OperandStackUnderflow,

    #[error("Runtime stack underflow")]
    RuntimeStackUnderflow,

    #[error("PC out of bounds: {0}")]
    PcOutOfBounds(usize),

    #[error("Bad type: expected {expected}, found {found}")]
    BadType { expected: String, found: String },

    #[error("Illegal argument: {0}")]
    IllegalArgument(String),

    #[error("Arity and params mismatch: arity {arity}, found {params} params")]
    ArityParamsMismatch { arity: usize, params: usize },

    #[error("Insufficient arguments: expected {expected}, got {got}")]
    InsufficientArguments { expected: usize, got: usize },

    #[error("Unknown builtin: {sym}")]
    UnknownBuiltin { sym: String },

    #[error("Unimplemented")]
    Unimplemented,
}
