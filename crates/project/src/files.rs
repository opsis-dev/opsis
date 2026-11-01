use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use opsis_compile::text::project_folder_name;
use opsis_engine::files::{atomic_write, none_if_missing, read_text_if_present};
use opsis_engine::hash::sha256_hex;
use opsis_types::project::{META_DIR, SaveResult};
use opsis_types::values::LastRun;
use opsis_types::workflow::{WORKFLOW_FILE, Workflow};

use crate::last_run::read_last_run;
use crate::paths::project_paths;
use crate::templates::build_template;

const VERSION_LENGTH: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("There's no Opsis project in {}.", .0.display())]
    NoProject(PathBuf),
    #[error("The project in {} can't be read ({problem}).", .dir.display())]
    Unreadable {
        dir: PathBuf,
        problem: WorkflowTextError,
    },
    #[error("Choose a full folder path for the project.")]
    NotAbsolute,
    #[error("{} already contains a project.", .0.display())]
    AlreadyAProject(PathBuf),
    #[error("There's no template called {0}.")]
    NoTemplate(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct ProjectFile {
    pub text: String,
    pub version: String,
}

#[derive(Debug)]
pub struct OpenedProject {
    pub workflow: Workflow,
    pub last_run: Option<LastRun>,
    pub version: String,
}

#[derive(Debug)]
pub struct CreatedProject {
    pub dir: PathBuf,
    pub workflow: Workflow,
    pub version: String,
}

pub fn version_of(text: &str) -> String {
    sha256_hex(text.as_bytes(), VERSION_LENGTH)
}

pub fn read_project_file(dir: &Path) -> std::io::Result<Option<ProjectFile>> {
    Ok(
        read_text_if_present(&dir.join(WORKFLOW_FILE))?.map(|text| ProjectFile {
            version: version_of(&text),
            text,
        }),
    )
}

#[derive(Debug, thiserror::Error)]
pub enum WorkflowTextError {
    #[error("the file is damaged")]
    Damaged,
    #[error("{} at {}", .0.inner(), .0.path())]
    Shape(serde_path_to_error::Error<serde_json::Error>),
}

pub fn parse_workflow_text(text: &str) -> Result<Workflow, WorkflowTextError> {
    let mut json = serde_json::Deserializer::from_str(text);
    match serde_path_to_error::deserialize::<_, Workflow>(&mut json) {
        Ok(workflow) => Ok(workflow.normalized()),
        Err(error) if error.inner().is_syntax() || error.inner().is_eof() => {
            Err(WorkflowTextError::Damaged)
        }
        Err(error) => Err(WorkflowTextError::Shape(error)),
    }
}

pub fn workflow_text(workflow: &Workflow) -> String {
    let text =
        serde_json::to_string_pretty(&workflow.clone().normalized()).expect("workflows serialize");
    format!("{text}\n")
}

pub fn clear_cache(dir: &Path) -> std::io::Result<()> {
    let paths = project_paths(dir);
    none_if_missing(std::fs::remove_dir_all(paths.cache))?;
    none_if_missing(std::fs::remove_file(paths.last_run))?;
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFiles {
    written: Arc<Mutex<HashMap<PathBuf, String>>>,
}

impl ProjectFiles {
    pub fn written_version(&self, dir: &Path) -> Option<String> {
        self.written
            .lock()
            .expect("written versions")
            .get(dir)
            .cloned()
    }

    fn remember(&self, dir: &Path, version: &str) {
        self.written
            .lock()
            .expect("written versions")
            .insert(dir.to_path_buf(), version.to_owned());
    }

    pub fn open(&self, dir: &Path) -> Result<OpenedProject, ProjectError> {
        let file =
            read_project_file(dir)?.ok_or_else(|| ProjectError::NoProject(dir.to_path_buf()))?;
        let workflow =
            parse_workflow_text(&file.text).map_err(|problem| ProjectError::Unreadable {
                dir: dir.to_path_buf(),
                problem,
            })?;
        self.remember(dir, &file.version);
        Ok(OpenedProject {
            workflow,
            last_run: read_last_run(dir)?,
            version: file.version,
        })
    }

    fn write(&self, dir: &Path, workflow: &Workflow) -> std::io::Result<String> {
        let text = workflow_text(workflow);
        let version = version_of(&text);
        self.remember(dir, &version);
        atomic_write(&dir.join(WORKFLOW_FILE), text.as_bytes())?;
        Ok(version)
    }

    pub fn create(
        &self,
        parent: &Path,
        name: &str,
        template: &str,
    ) -> Result<CreatedProject, ProjectError> {
        if !parent.is_absolute() {
            return Err(ProjectError::NotAbsolute);
        }
        let dir = parent.join(project_folder_name(name));
        if dir.join(WORKFLOW_FILE).exists() {
            return Err(ProjectError::AlreadyAProject(dir));
        }
        let workflow = build_template(template, name)
            .ok_or_else(|| ProjectError::NoTemplate(template.to_owned()))?;
        std::fs::create_dir_all(project_paths(&dir).assets)?;
        std::fs::write(dir.join(".gitignore"), format!("{META_DIR}/\n"))?;
        let version = self.write(&dir, &workflow)?;
        Ok(CreatedProject {
            dir,
            workflow,
            version,
        })
    }

    pub fn save(
        &self,
        dir: &Path,
        workflow: &Workflow,
        base: Option<&str>,
    ) -> std::io::Result<SaveResult> {
        if let Some(current) = read_project_file(dir)?
            && Some(current.version.as_str()) != base
        {
            return Ok(SaveResult::Changed {
                workflow: parse_workflow_text(&current.text).ok().map(Box::new),
                version: current.version,
            });
        }
        Ok(SaveResult::Saved {
            version: self.write(dir, workflow)?,
        })
    }
}

pub fn expand_home(folder: &str, home: &Path) -> PathBuf {
    PathBuf::from(
        shellexpand::tilde_with_context(folder, || Some(home.to_string_lossy())).into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn made(files: &ProjectFiles, parent: &Path, name: &str) -> CreatedProject {
        files.create(parent, name, "blank").unwrap()
    }

    #[test]
    fn creates_a_project_folder_named_after_the_project_and_opens_it_again() {
        let parent = tempfile::tempdir().unwrap();
        let files = ProjectFiles::default();
        let created = made(&files, parent.path(), "Launch video");
        assert_eq!(created.dir, parent.path().join("launch-video"));
        assert_eq!(
            std::fs::read_to_string(created.dir.join(".gitignore")).unwrap(),
            ".opsis/\n"
        );
        assert!(created.dir.join("assets").is_dir());
        let opened = files.open(&created.dir).unwrap();
        assert_eq!(
            (opened.workflow, opened.version.as_str()),
            (created.workflow, created.version.as_str())
        );
        assert_eq!(files.written_version(&created.dir), Some(created.version));
        assert!(matches!(
            files.create(parent.path(), "Launch video", "blank"),
            Err(ProjectError::AlreadyAProject(_))
        ));
        assert!(matches!(
            files.create(Path::new("relative"), "x", "blank"),
            Err(ProjectError::NotAbsolute)
        ));
    }

    #[test]
    fn refuses_to_save_over_a_file_changed_on_disk_and_returns_what_is_there() {
        let parent = tempfile::tempdir().unwrap();
        let files = ProjectFiles::default();
        let created = made(&files, parent.path(), "Demo");
        let mut edited = created.workflow.clone();
        edited.name = "Edited".into();
        let SaveResult::Saved { version } = files
            .save(&created.dir, &edited, Some(&created.version))
            .unwrap()
        else {
            panic!("the first save writes")
        };
        let mut outside = edited.clone();
        outside.name = "From outside".into();
        std::fs::write(created.dir.join(WORKFLOW_FILE), workflow_text(&outside)).unwrap();
        let refused = files.save(&created.dir, &edited, Some(&version)).unwrap();
        assert_eq!(
            refused,
            SaveResult::Changed {
                version: version_of(&workflow_text(&outside)),
                workflow: Some(Box::new(outside.normalized()))
            }
        );
    }

    #[test]
    fn opens_every_example_as_the_templates_write_it_today() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
        let files = ProjectFiles::default();
        for (id, workflow) in crate::templates::examples() {
            let dir = root.join(&id);
            let opened = files
                .open(&dir)
                .unwrap_or_else(|e| panic!("{e} Run `bun run examples`."));
            assert_eq!(
                std::fs::read_to_string(dir.join(WORKFLOW_FILE)).unwrap(),
                workflow_text(&workflow),
                "examples/{id} is stale. Run `bun run examples`."
            );
            assert_eq!(opened.workflow, workflow.normalized());
        }
    }

    #[test]
    fn says_why_a_project_cant_be_opened() {
        let dir = tempfile::tempdir().unwrap();
        let files = ProjectFiles::default();
        assert!(matches!(
            files.open(dir.path()),
            Err(ProjectError::NoProject(_))
        ));
        std::fs::write(dir.path().join(WORKFLOW_FILE), "{").unwrap();
        assert_eq!(
            files.open(dir.path()).err().map(|e| e.to_string()),
            Some(format!(
                "The project in {} can't be read (the file is damaged).",
                dir.path().display()
            ))
        );
        std::fs::write(
            dir.path().join(WORKFLOW_FILE),
            r#"{"version":1,"nodes":"none"}"#,
        )
        .unwrap();
        let problem = files
            .open(dir.path())
            .err()
            .map_or_default(|e| e.to_string());
        assert!(
            problem.contains("can't be read (") && !problem.contains("damaged"),
            "{problem}"
        );
    }

    #[test]
    fn clearing_the_cache_forgets_the_last_run_and_keeps_the_rest() {
        let dir = tempfile::tempdir().unwrap();
        let paths = project_paths(dir.path());
        std::fs::create_dir_all(paths.pictures.clone()).unwrap();
        std::fs::write(&paths.last_run, "{}").unwrap();
        std::fs::create_dir_all(&paths.traces).unwrap();
        clear_cache(dir.path()).unwrap();
        clear_cache(dir.path()).unwrap();
        assert!(!paths.cache.exists() && !paths.last_run.exists() && paths.traces.exists());
    }

    #[test]
    fn expands_a_leading_tilde_to_the_home_folder() {
        let home = Path::new("/home/someone");
        assert_eq!(expand_home("~", home), home);
        assert_eq!(expand_home("~/Videos", home), home.join("Videos"));
        assert_eq!(expand_home("/srv/~x", home), Path::new("/srv/~x"));
    }
}
