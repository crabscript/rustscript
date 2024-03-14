use thiserror::Error;

#[derive(Error, Debug)]
pub enum ByteCodeError {
    #[error("Type mismatch, expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
}
