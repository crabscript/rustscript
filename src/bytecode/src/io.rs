#![allow(dead_code)] // TODO: remove this line once the module is used
use super::ByteCode;
use anyhow::Result;
use std::io::{Read, Write};

/// Serialize the bytecode to the writer.
/// The serialized format is:
/// - 8 bytes for the length of the serialized bytecode
/// - The serialized bytecode
///
/// # Arguments
/// - `bytecode`: The bytecode to serialize
/// - `writer`: The writer to write the serialized bytecode to
///
/// # Returns
/// - `Result<()>`: The result of the serialization
pub fn write_bytecode<W: Write>(bytecode: &[ByteCode], writer: &mut W) -> Result<()> {
    let serialized = bincode::serialize(bytecode)?;
    let len = serialized.len() as u64;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&serialized)?;
    Ok(())
}

/// Deserialize the bytecode from the reader.
/// The serialized format is:
/// - 8 bytes for the length of the serialized bytecode
/// - The serialized bytecode
///
/// # Arguments
/// - `reader`: The reader to read the serialized bytecode from
///
/// # Returns
/// - `Result<Vec<ByteCode>>`: The result of the deserialization
pub fn read_bytecode<R: Read>(reader: &mut R) -> Result<Vec<ByteCode>> {
    let mut len_bytes = [0; 8];
    reader.read_exact(&mut len_bytes)?;
    let len = u64::from_le_bytes(len_bytes) as usize;
    let mut serialized = vec![0; len];
    reader.read_exact(&mut serialized)?;
    let bytecode = bincode::deserialize(&serialized)?;
    Ok(bytecode)
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    #[test]
    fn test_deterministic_serialization() {
        let bc = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(42.0),
            ByteCode::BINOP(BinOp::Add),
            ByteCode::UNOP(UnOp::Neg),
        ];
        let mut serialized = Vec::new();
        write_bytecode(&bc, &mut serialized).unwrap();
        let deserialized = read_bytecode(&mut serialized.as_slice()).unwrap();
        assert_eq!(bc, deserialized);
    }

    #[test]
    fn test_deterministic_serialization_file() {
        let bc = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(42.0),
            ByteCode::BINOP(BinOp::Add),
            ByteCode::UNOP(UnOp::Neg),
            ByteCode::GOTO(6),
            ByteCode::JOF(0),
            ByteCode::DONE,
        ];

        let mut file = std::fs::File::create("test.o2").unwrap();
        write_bytecode(&bc, &mut file).unwrap();
        file.sync_all().unwrap();

        // read from file
        let mut file = std::fs::File::open("test.o2").unwrap();
        let deserialized = read_bytecode(&mut file).unwrap();
        assert_eq!(bc, deserialized);

        // remove file
        std::fs::remove_file("test.o2").unwrap();
    }
}
