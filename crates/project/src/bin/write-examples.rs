use std::path::PathBuf;

use opsis_project::files::workflow_text;
use opsis_project::templates::{build_template, templates};

const BLANK: &str = "blank";

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for template in templates().into_iter().filter(|t| t.id != BLANK) {
        let mut workflow =
            build_template(&template.id, &template.title).expect("a listed template");
        workflow.id = format!("example_{}", template.id.replace('-', "_"));
        let dir = root.join("examples").join(&template.id);
        std::fs::create_dir_all(dir.join("assets")).expect("the example folder");
        std::fs::write(dir.join("assets").join(".gitkeep"), "").expect("the assets folder");
        std::fs::write(dir.join("project.opsis.json"), workflow_text(&workflow))
            .expect("the project file");
        std::fs::write(dir.join(".gitignore"), ".opsis/\nsecrets/\n").expect("the ignore file");
        println!(
            "wrote examples/{}/project.opsis.json ({} nodes)",
            template.id,
            workflow.nodes.len()
        );
    }
}
