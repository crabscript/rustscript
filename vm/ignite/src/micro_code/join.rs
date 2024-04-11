use anyhow::Result;
use bytecode::ThreadID;

use crate::Runtime;

pub fn join(rt: &mut Runtime, tid: ThreadID) -> Result<()> {
    Ok(())
}
