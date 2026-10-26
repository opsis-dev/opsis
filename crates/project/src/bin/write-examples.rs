use std::path::PathBuf;

use opsis_project::files::workflow_text;
use opsis_project::templates::examples;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    for (id, workflow) in examples() {
        let dir = root.join(&id);
        std::fs::create_dir_all(dir.join("assets")).expect("the example folder");
        std::fs::write(dir.join("assets").join(".gitkeep"), "").expect("the assets folder");
        std::fs::write(dir.join("project.opsis.json"), workflow_text(&workflow))
            .expect("the project file");
        std::fs::write(dir.join(".gitignore"), ".opsis/\nsecrets/\n").expect("the ignore file");
        println!(
            "wrote examples/{id}/project.opsis.json ({} nodes)",
            workflow.nodes.len()
        );
    }
}
