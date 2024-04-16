use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteCodeError {
    #[error("Type mismatch, expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Bad type, expected {expected}, found {found}")]
    BadType { expected: String, found: String },

    #[error("Unbounded name: {name}")]
    UnboundedName { name: String },

    #[error("Environment access after drop")]
    EnvironmentDroppedError,
}
