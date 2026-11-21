use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Duration;

use opsis_types::settings::OwnAgent;
use serde::Deserialize;

use crate::Session;
use crate::state::AppState;

const SNAPSHOT: &str = include_str!("agents.generated.json");
pub const BUNDLED_ID: &str = "opsis-agent";
pub const BUNDLED_NAME: &str = "Opsis agent";
const VERSION_WITHIN: Duration = Duration::from_secs(3);
const VERSION_LENGTH: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Launch {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub website: String,
    pub license: String,
    pub launches: Vec<Launch>,
}

#[derive(Debug, Deserialize)]
struct Snapshot {
    presets: Vec<Preset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Bundled,
    Registry,
    Own,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentChoice {
    pub id: String,
    pub name: String,
    pub description: String,
    pub command: PathBuf,
    pub args: Vec<String>,
    pub version: Option<String>,
    pub source: Source,
}

pub fn presets() -> Vec<Preset> {
    match serde_json::from_str::<Snapshot>(SNAPSHOT) {
        Ok(snapshot) => snapshot.presets,
        Err(error) => {
            log::error!("reading the agent list: {error}");
            Vec::new()
        }
    }
}

#[cfg(unix)]
fn runnable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn runnable(path: &Path) -> bool {
    path.is_file()
}

const WINDOWS_ENDINGS: [&str; 4] = ["exe", "cmd", "bat", "com"];

pub fn find_program(program: &str, path_var: &OsStr, windows: bool) -> Option<PathBuf> {
    std::env::split_paths(path_var).find_map(|dir| {
        let plain = dir.join(program);
        if !windows {
            return runnable(&plain).then_some(plain);
        }
        WINDOWS_ENDINGS
            .iter()
            .map(|ending| dir.join(format!("{program}.{ending}")))
            .find(|path| runnable(path))
    })
}

pub fn found(
    presets: &[Preset],
    path_var: &OsStr,
    windows: bool,
) -> Vec<(Preset, Launch, PathBuf)> {
    presets
        .iter()
        .filter_map(|preset| {
            preset.launches.iter().find_map(|launch| {
                find_program(&launch.program, path_var, windows)
                    .map(|path| (preset.clone(), launch.clone(), path))
            })
        })
        .collect()
}

pub fn first_line(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(VERSION_LENGTH).collect())
}

pub async fn version_of(command: &Path) -> Option<String> {
    let ran = tokio::time::timeout(
        VERSION_WITHIN,
        tokio::process::Command::new(command)
            .arg("--version")
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .ok()?
    .ok()?;
    if !ran.status.success() {
        return None;
    }
    first_line(&String::from_utf8_lossy(&ran.stdout))
}

pub fn bundled_command() -> PathBuf {
    let name = if cfg!(windows) {
        "opsis-agent.exe"
    } else {
        "opsis-agent"
    };
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn own_choices(own: &[OwnAgent]) -> Vec<AgentChoice> {
    own.iter()
        .map(|agent| AgentChoice {
            id: format!("own:{}", agent.name),
            name: agent.name.clone(),
            description: String::new(),
            command: PathBuf::from(&agent.command),
            args: agent.args.clone(),
            version: None,
            source: Source::Own,
        })
        .collect()
}

pub async fn available(app: &Session) -> Vec<AgentChoice> {
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    let bundled = bundled_command();
    let mut choices = vec![AgentChoice {
        id: BUNDLED_ID.to_owned(),
        name: BUNDLED_NAME.to_owned(),
        description: "Opsis's own agent, with the model service you log in to.".to_owned(),
        version: version_of(&bundled).await,
        command: bundled,
        args: Vec::new(),
        source: Source::Bundled,
    }];
    for (preset, launch, path) in found(&presets(), &path_var, cfg!(windows)) {
        choices.push(AgentChoice {
            id: preset.id,
            name: preset.name,
            description: preset.description,
            version: version_of(&path).await,
            command: path,
            args: launch.args,
            source: Source::Registry,
        });
    }
    let own = app.state::<AppState>().settings().assistant.agents;
    choices.extend(own_choices(&own));
    choices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_snapshot_reads_and_lists_agents_with_plain_program_names() {
        let presets = presets();
        assert!(presets.len() > 10);
        for preset in &presets {
            assert!(!preset.launches.is_empty(), "{}", preset.id);
            for launch in &preset.launches {
                assert!(!launch.program.contains(['/', '\\']), "{}", launch.program);
            }
        }
    }

    fn preset(id: &str, programs: &[&str]) -> Preset {
        Preset {
            id: id.to_owned(),
            name: id.to_owned(),
            version: "1.0.0".to_owned(),
            description: String::new(),
            website: String::new(),
            license: String::new(),
            launches: programs
                .iter()
                .map(|program| Launch {
                    program: (*program).to_owned(),
                    args: vec!["--acp".to_owned()],
                })
                .collect(),
        }
    }

    #[cfg(unix)]
    fn program(dir: &Path, name: &str, runs: bool) {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, "#!/bin/sh\necho \"$0 2.4.1\"\n").unwrap();
        let mode = if runs { 0o755 } else { 0o644 };
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn finds_only_agents_whose_program_runs_on_the_path_first_launch_first() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        program(first.path(), "claude-agent-acp", true);
        program(first.path(), "codex-acp", false);
        program(second.path(), "goose", true);
        program(second.path(), "goose-acp", true);
        let path_var = std::env::join_paths([first.path(), second.path()]).unwrap();
        let presets = [
            preset("claude-acp", &["claude-agent-acp"]),
            preset("codex-acp", &["codex-acp"]),
            preset("goose", &["goose-acp", "goose"]),
            preset("missing", &["nothing-here"]),
        ];
        let found: Vec<(String, String)> = found(&presets, &path_var, false)
            .into_iter()
            .map(|(preset, launch, _)| (preset.id, launch.program))
            .collect();
        assert_eq!(
            found,
            [
                ("claude-acp".to_owned(), "claude-agent-acp".to_owned()),
                ("goose".to_owned(), "goose-acp".to_owned())
            ]
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn reads_a_version_from_the_programs_first_line() {
        let dir = tempfile::tempdir().unwrap();
        program(dir.path(), "agent", true);
        let version = version_of(&dir.path().join("agent")).await.unwrap();
        assert!(version.ends_with("2.4.1"), "{version}");
        assert_eq!(version_of(&dir.path().join("absent")).await, None);
    }

    #[test]
    fn own_commands_come_as_they_were_written() {
        let own = [OwnAgent {
            name: "My agent".to_owned(),
            command: "/opt/agent/bin/run".to_owned(),
            args: vec!["acp".to_owned()],
        }];
        let choices = own_choices(&own);
        assert_eq!(choices[0].command, PathBuf::from("/opt/agent/bin/run"));
        assert_eq!(choices[0].source, Source::Own);
    }

    #[test]
    fn a_version_is_the_first_line_with_text_cut_to_a_fixed_length() {
        assert_eq!(
            first_line("\n  codex 1.2\nmore"),
            Some("codex 1.2".to_owned())
        );
        assert_eq!(
            first_line(&"x".repeat(200)).map(|v| v.len()),
            Some(VERSION_LENGTH)
        );
        assert_eq!(first_line("  \n"), None);
    }
}
