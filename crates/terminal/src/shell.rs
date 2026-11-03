use std::ffi::OsString;
use std::path::{Path, PathBuf};

use opsis_types::terminal::{EnvVar, ShellChoice};

const BASH_RC: &str = include_str!("integration/bash.sh");
const ZSH_RC: &str = include_str!("integration/zshrc");
const POWERSHELL_PROFILE: &str = include_str!("integration/profile.ps1");
const BASH_RC_FILE: &str = "opsis-bashrc";
const ZSH_FOLDER: &str = "opsis-zsh";
const POWERSHELL_FILE: &str = "opsis-profile.ps1";

#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("{0} isn't installed")]
    Missing(String),
    #[error("Couldn't set up the shell: {0}")]
    Setup(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: PathBuf,
    pub marks: bool,
}

pub fn find_program(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.components().count() > 1 {
        return path.is_file().then(|| path.to_path_buf());
    }
    let dirs = std::env::var_os("PATH")?;
    let extensions: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_owned())
            .split(';')
            .map(str::to_lowercase)
            .chain(std::iter::once(String::new()))
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&dirs).find_map(|dir| {
        extensions.iter().find_map(|ext| {
            let candidate = dir.join(format!("{name}{ext}"));
            candidate.is_file().then_some(candidate)
        })
    })
}

const fn locale() -> &'static str {
    if cfg!(target_os = "macos") {
        "en_US.UTF-8"
    } else {
        "C.UTF-8"
    }
}

fn base_env(temp: &Path, vars: &[EnvVar]) -> Env {
    let mut env: Env = vec![
        ("TERM".into(), "xterm-256color".into()),
        ("COLORTERM".into(), "truecolor".into()),
        ("LANG".into(), locale().into()),
        ("LC_ALL".into(), locale().into()),
    ];
    for name in ["TMPDIR", "TEMP", "TMP"] {
        env.push((name.into(), temp.as_os_str().to_owned()));
    }
    for var in vars.iter().filter(|v| !v.name.trim().is_empty()) {
        env.push((var.name.trim().into(), var.value.clone().into()));
    }
    env
}

fn powershell() -> Result<PathBuf, ShellError> {
    find_program("pwsh")
        .or_else(|| find_program("powershell"))
        .ok_or_else(|| ShellError::Missing("PowerShell".to_owned()))
}

type Env = Vec<(OsString, OsString)>;

struct Launch {
    program: PathBuf,
    args: Vec<OsString>,
    env: Env,
}

fn clean(temp: &Path) -> Result<Launch, ShellError> {
    if cfg!(windows) {
        let profile = temp.join(POWERSHELL_FILE);
        std::fs::write(&profile, POWERSHELL_PROFILE)?;
        let command = format!(". '{}'", profile.display());
        let args = ["-NoLogo", "-NoProfile", "-NoExit", "-Command", &command]
            .map(OsString::from)
            .to_vec();
        return Ok(Launch {
            program: powershell()?,
            args,
            env: Vec::new(),
        });
    }
    if cfg!(target_os = "macos") {
        let dir = temp.join(ZSH_FOLDER);
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join(".zshrc"), ZSH_RC)?;
        let zsh = find_program("zsh").ok_or_else(|| ShellError::Missing("zsh".to_owned()))?;
        return Ok(Launch {
            program: zsh,
            args: vec!["-d".into(), "-i".into()],
            env: vec![("ZDOTDIR".into(), dir.into_os_string())],
        });
    }
    let rc = temp.join(BASH_RC_FILE);
    std::fs::write(&rc, BASH_RC)?;
    let bash = find_program("bash").ok_or_else(|| ShellError::Missing("bash".to_owned()))?;
    Ok(Launch {
        program: bash,
        args: vec![
            "--noprofile".into(),
            "--rcfile".into(),
            rc.into_os_string(),
            "-i".into(),
        ],
        env: Vec::new(),
    })
}

fn own() -> Result<(PathBuf, Vec<OsString>), ShellError> {
    if cfg!(windows) {
        return Ok((powershell()?, vec!["-NoLogo".into()]));
    }
    let shell = std::env::var_os("SHELL")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| find_program("sh"))
        .ok_or_else(|| ShellError::Missing("A shell".to_owned()))?;
    Ok((shell, vec!["-i".into()]))
}

pub fn shell_command(
    choice: ShellChoice,
    cwd: &Path,
    temp: &Path,
    vars: &[EnvVar],
) -> Result<ShellCommand, ShellError> {
    let mut env = base_env(temp, vars);
    let (program, args, marks) = match choice {
        ShellChoice::Clean => {
            let launch = clean(temp)?;
            env.extend(launch.env);
            (launch.program, launch.args, true)
        }
        ShellChoice::Own => {
            let (program, args) = own()?;
            (program, args, false)
        }
    };
    Ok(ShellCommand {
        program,
        args,
        env,
        cwd: cwd.to_path_buf(),
        marks,
    })
}

pub fn require(program: &str) -> Result<(), ShellError> {
    find_program(program.trim())
        .map(|_| ())
        .ok_or_else(|| ShellError::Missing(program.trim().to_owned()))
}
