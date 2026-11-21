use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use opsis_packager::appimage::AppImageTools;
use opsis_packager::{AppFiles, Format, PackagerError, Tools, package};

#[derive(Debug, Parser)]
#[command(name = "opsis-packager", version)]
#[doc = "Builds Opsis's installers with each format's own tool, from programs already built."]
struct Args {
    #[arg(long, value_name = "PATH")]
    #[doc = "The opsis program"]
    program: PathBuf,
    #[arg(long, value_name = "PATH")]
    #[doc = "The ffmpeg program to bundle"]
    ffmpeg: PathBuf,
    #[arg(long, value_name = "PATH")]
    #[doc = "The opsis-renderer program"]
    renderer: PathBuf,
    #[arg(long, value_name = "PATH")]
    #[doc = "The opsis-mcp program"]
    mcp: PathBuf,
    #[arg(long, value_name = "PATH")]
    #[doc = "The opsis-agent program"]
    agent: PathBuf,
    #[arg(long, value_name = "TRIPLE")]
    #[doc = "The Rust target the programs were built for"]
    target_triple: String,
    #[arg(long, value_name = "REPOSITORY")]
    #[doc = "The repository's root, for the resources, icons and notices"]
    root: PathBuf,
    #[arg(long, value_name = "FOLDER")]
    #[doc = "Where to write the installers"]
    out: PathBuf,
    #[arg(long, value_name = "FORMAT", value_delimiter = ',', required = true)]
    #[doc = "The installers to build, separated by commas: deb, rpm, pacman, appimage, nsis, msi, app, dmg"]
    formats: Vec<String>,
    #[arg(long, value_name = "PATH", requires = "appimage_runtime")]
    #[doc = "The appimagetool program, for the appimage format"]
    appimagetool: Option<PathBuf>,
    #[arg(long, value_name = "PATH", requires = "appimagetool")]
    #[doc = "The runtime an appimage starts with"]
    appimage_runtime: Option<PathBuf>,
}

fn run(args: Args) -> Result<Vec<PathBuf>, PackagerError> {
    let app = AppFiles {
        program: args.program,
        ffmpeg: args.ffmpeg,
        renderer: args.renderer,
        mcp: args.mcp,
        agent: args.agent,
        target_triple: args.target_triple,
        root: args.root,
    };
    let formats = args
        .formats
        .iter()
        .map(|format| Format::parse(format))
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
    package(&app, &formats, &tools, &args.out)
}

fn main() -> ExitCode {
    match run(Args::parse()) {
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
