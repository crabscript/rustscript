use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("stack underflow")]
    StackUnderflow,

    #[error("bad type: expected {expected}, found {found}")]
    BadType { expected: String, found: String },

    #[error("bad argument: {0}")]
    IllegalArgument(String),

    #[error("unimplemented")]
    Unimplemented,

    #[error("generic error: {0}")]
    Generic(String),
}
