use anyhow::Result;

pub use crate::error::*;
use crate::runtime::Runtime;

mod error;
mod micro_code;
mod runtime;

fn main() -> Result<()> {
    let rt = Runtime::new(vec![]);
    runtime::run(rt)?;
    Ok(())
}
