use opsis_project::files::workflow_text;
use opsis_project::templates::examples;

use crate::error::ToolError;
use crate::output;

const EXAMPLES_DIR: &str = "examples";
const IGNORED: &str = ".opsis/\nsecrets/\n";

pub fn write() -> Result<(), ToolError> {
    let root = output::repository().join(EXAMPLES_DIR);
    for (id, workflow) in examples() {
        let dir = root.join(&id);
        output::write(&dir.join("assets").join(".gitkeep"), "")?;
        output::write(&dir.join(".gitignore"), IGNORED)?;
        let project = dir.join("project.opsis.json");
        output::write(&project, workflow_text(&workflow))?;
        println!(
            "wrote {} ({} nodes)",
            output::shown(&project),
            workflow.nodes.len()
        );
    }
    Ok(())
}
