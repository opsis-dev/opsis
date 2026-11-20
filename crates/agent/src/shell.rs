use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use rig_core::completion::{ToolAnnotations, ToolDefinition};
use rig_core::message::ToolName;
use serde_json::json;
use tokio::io::AsyncReadExt;

pub const SHELL: &str = "shell";
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
pub const MAX_TIMEOUT_SECONDS: u64 = 1800;
const KEPT_BYTES: usize = 30_000;
const CHAINING: [&str; 10] = [";", "&", "|", ">", "<", "`", "$(", "\n", "\r", "${"];
const READING_PROGRAMS: [&str; 19] = [
    "ls", "cat", "head", "tail", "wc", "rg", "grep", "find", "pwd", "file", "stat", "du", "tree",
    "which", "echo", "realpath", "dirname", "basename", "fd",
];
const READING_GIT: [&str; 9] = [
    "status",
    "diff",
    "log",
    "show",
    "ls-files",
    "blame",
    "rev-parse",
    "describe",
    "shortlog",
];
const WRITING_FIND: [&str; 7] = [
    "-exec", "-execdir", "-ok", "-okdir", "-delete", "-fprint", "-fls",
];

pub fn definition() -> Option<ToolDefinition> {
    Some(ToolDefinition {
        name: ToolName::new(SHELL).ok()?,
        description: "Run a shell command in the session's folder and answer with its output and exit code. With background, it starts and its result comes back as a message when it ends.".to_owned(),
        parameters: json!({ "type": "object", "properties": {
            "command": { "type": "string" },
            "background": { "type": "boolean" },
            "timeout_seconds": { "type": "integer", "minimum": 1, "maximum": MAX_TIMEOUT_SECONDS }
        }, "required": ["command"] }),
        annotations: ToolAnnotations::default(),
    })
}

pub fn reads_only(command: &str) -> bool {
    if CHAINING.iter().any(|token| command.contains(token)) {
        return false;
    }
    let words: Vec<&str> = command.split_whitespace().collect();
    let Some((program, rest)) = words.split_first() else {
        return false;
    };
    match *program {
        "git" => {
            rest.first().is_some_and(|sub| READING_GIT.contains(sub)) && !rest.contains(&"--output")
        }
        "find" => !rest.iter().any(|word| WRITING_FIND.contains(word)),
        "rg" | "fd" => !rest.iter().any(|word| {
            word.starts_with("--pre")
                || *word == "-x"
                || *word == "--exec"
                || *word == "-X"
                || *word == "--exec-batch"
        }),
        program => READING_PROGRAMS.contains(&program),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    pub code: Option<i32>,
    pub output: String,
    pub timed_out: bool,
}

impl Ran {
    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }

    pub fn report(&self) -> String {
        let status = match (self.timed_out, self.code) {
            (true, _) => "It was stopped after its time ran out.".to_owned(),
            (false, Some(code)) => format!("Exit code {code}."),
            (false, None) => "It ended without an exit code.".to_owned(),
        };
        if self.output.is_empty() {
            status
        } else {
            format!("{status}\n{}", self.output)
        }
    }
}

fn kept(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= KEPT_BYTES {
        return text.into_owned();
    }
    let mut start = text.len() - KEPT_BYTES;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    format!("…{}", &text[start..])
}

fn command(line: &str, folder: &Path) -> tokio::process::Command {
    let mut command = if cfg!(windows) {
        let mut command = tokio::process::Command::new("cmd");
        command.args(["/C", line]);
        command
    } else {
        let mut command = tokio::process::Command::new("sh");
        command.args(["-c", line]);
        command
    };
    command
        .current_dir(folder)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    command
}

pub async fn run(line: &str, folder: &Path, timeout: Duration) -> std::io::Result<Ran> {
    let mut child = command(line, folder).spawn()?;
    let mut stdout = child.stdout.take();
    let mut stderr = child.stderr.take();
    let read = async {
        let mut out = Vec::new();
        let mut err = Vec::new();
        if let Some(stdout) = stdout.as_mut() {
            stdout.read_to_end(&mut out).await?;
        }
        if let Some(stderr) = stderr.as_mut() {
            stderr.read_to_end(&mut err).await?;
        }
        let status = child.wait().await?;
        out.extend(err);
        Ok::<_, std::io::Error>((status.code(), out))
    };
    match tokio::time::timeout(timeout, read).await {
        Ok(read) => {
            let (code, output) = read?;
            Ok(Ran {
                code,
                output: kept(&output),
                timed_out: false,
            })
        }
        Err(_) => Ok(Ran {
            code: None,
            output: String::new(),
            timed_out: true,
        }),
    }
}

pub fn timeout_of(seconds: Option<u64>) -> Duration {
    Duration::from_secs(
        seconds
            .unwrap_or(DEFAULT_TIMEOUT_SECONDS)
            .clamp(1, MAX_TIMEOUT_SECONDS),
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn reading() -> impl Strategy<Value = String> {
        prop_oneof![
            "ls( -la)?( [a-z/]{1,10})?",
            "cat [a-z]{1,8}\\.lua",
            "rg [a-z]{1,8}( [a-z/]{1,8})?",
            "git (status|diff|log --oneline|show HEAD)",
            "head -n [0-9]{1,3} [a-z]{1,8}",
        ]
    }

    proptest! {
        #[test]
        fn plain_reading_commands_run_without_asking(command in reading()) {
            prop_assert!(reads_only(&command));
        }

        #[test]
        fn chaining_or_redirecting_a_reading_command_always_asks(
            command in reading(),
            joint in prop::sample::select(vec![";", "&&", "||", "|", ">", ">>", "<", "&", "`", "$(", "\n"]),
            tail in "[a-z ]{0,12}",
        ) {
            let line = format!("{command}{joint}{tail}");
            prop_assert!(!reads_only(&line));
        }

        #[test]
        fn a_writing_program_always_asks(
            program in prop::sample::select(vec!["rm", "mv", "cp", "touch", "mkdir", "chmod", "sed", "tee", "dd", "curl", "npm", "cargo", "python3", "sh", "bash"]),
            args in "[a-z -]{0,16}",
        ) {
            let line = format!("{program} {args}");
            prop_assert!(!reads_only(&line));
        }

        #[test]
        fn find_and_git_that_change_things_always_ask(
            target in "[a-z]{1,8}",
            action in prop::sample::select(vec!["-delete", "-exec rm {} +", "-execdir touch x +", "-ok rm {} +"]),
            sub in prop::sample::select(vec!["commit", "push", "checkout", "reset", "add", "rm", "stash", "clean"]),
        ) {
            let find = format!("find {target} {action}");
            let git = format!("git {sub} {target}");
            prop_assert!(!reads_only(&find));
            prop_assert!(!reads_only(&git));
        }
    }

    #[tokio::test]
    async fn runs_a_command_in_the_folder_and_reports_its_output_and_code() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let line = if cfg!(windows) {
            "type a.txt"
        } else {
            "cat a.txt; exit 3"
        };
        let ran = run(line, dir.path(), Duration::from_secs(10))
            .await
            .unwrap();
        assert!(ran.output.contains("hello"));
        if !cfg!(windows) {
            assert_eq!(ran.code, Some(3));
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn a_command_past_its_time_is_stopped() {
        let dir = tempfile::tempdir().unwrap();
        let ran = run("sleep 5", dir.path(), Duration::from_millis(200))
            .await
            .unwrap();
        assert!(ran.timed_out);
    }
}
