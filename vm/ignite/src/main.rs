pub use crate::error::*;
pub use crate::runtime::Runtime;
use anyhow::Result;

mod error;
mod micro_code;
mod runtime;

fn main() -> Result<()> {
    let rt = Runtime::new(vec![]);
    runtime::run(rt)?;
    Ok(())
}
