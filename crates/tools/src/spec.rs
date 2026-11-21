use opsis_compile::ts_export::{SPEC_TYPES_FILE, spec_types};

use crate::error::ToolError;
use crate::output;

pub fn export() -> Result<(), ToolError> {
    let path = output::repository().join(SPEC_TYPES_FILE);
    output::write(&path, spec_types())?;
    println!("wrote {}", output::shown(&path));
    Ok(())
}
