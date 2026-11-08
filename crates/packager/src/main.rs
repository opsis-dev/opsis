use std::path::PathBuf;
use std::process::ExitCode;

use opsis_packager::appimage::AppImageTools;
use opsis_packager::{AppFiles, Format, PackagerError, Tools, package};

const USAGE: &str = "opsis-packager --program <path> --ffmpeg <path> --target-triple <triple> --root <repository> --out <folder> --formats <deb,rpm,…> [--appimagetool <path> --appimage-runtime <path>]";

#[derive(Debug, Default)]
struct Args {
    program: Option<PathBuf>,
    ffmpeg: Option<PathBuf>,
    target_triple: Option<String>,
    root: Option<PathBuf>,
    out: Option<PathBuf>,
    formats: Option<String>,
    appimagetool: Option<PathBuf>,
    appimage_runtime: Option<PathBuf>,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, PackagerError> {
    let mut parsed = Args::default();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| PackagerError::Usage(format!("{flag} needs a value; {USAGE}")))?;
        match flag.as_str() {
            "--program" => parsed.program = Some(value.into()),
            "--ffmpeg" => parsed.ffmpeg = Some(value.into()),
            "--target-triple" => parsed.target_triple = Some(value),
            "--root" => parsed.root = Some(value.into()),
            "--out" => parsed.out = Some(value.into()),
            "--formats" => parsed.formats = Some(value),
            "--appimagetool" => parsed.appimagetool = Some(value.into()),
            "--appimage-runtime" => parsed.appimage_runtime = Some(value.into()),
            _ => {
                return Err(PackagerError::Usage(format!(
                    "unknown flag {flag}; {USAGE}"
                )));
            }
        }
    }
    Ok(parsed)
}

fn needed<T>(value: Option<T>, flag: &str) -> Result<T, PackagerError> {
    value.ok_or_else(|| PackagerError::Usage(format!("{flag} is needed; {USAGE}")))
}

fn run() -> Result<Vec<PathBuf>, PackagerError> {
    let args = parse_args(std::env::args().skip(1))?;
    let app = AppFiles {
        program: needed(args.program, "--program")?,
        ffmpeg: needed(args.ffmpeg, "--ffmpeg")?,
        target_triple: needed(args.target_triple, "--target-triple")?,
        root: needed(args.root, "--root")?,
    };
    let formats = needed(args.formats, "--formats")?
        .split(',')
        .map(Format::parse)
        .collect::<Result<Vec<_>, _>>()?;
    let tools = Tools {
        appimage: args
            .appimagetool
            .zip(args.appimage_runtime)
            .map(|(appimagetool, runtime)| AppImageTools {
                appimagetool,
                runtime,
            }),
    };
    package(&app, &formats, &tools, &needed(args.out, "--out")?)
}

fn main() -> ExitCode {
    match run() {
        Ok(made) => {
            for path in made {
                println!("{}", path.display());
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
