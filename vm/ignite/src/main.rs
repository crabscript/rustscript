use anyhow::Result;

pub use crate::error::*;
use crate::virtual_machine::Runtime;

mod error;
mod virtual_machine;

fn main() -> Result<()> {
    let rt = Runtime::new(vec![]);
    virtual_machine::run(rt)?;
    Ok(())
}
