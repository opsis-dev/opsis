mod catalog;
mod engine;
mod error;
mod examples;
mod output;
mod spec;
mod stills;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use crate::error::ToolError;

#[derive(Debug, Parser)]
#[command(name = "opsis-tools", version)]
#[doc = "The development tools Opsis's scripts call: everything that needs the app's own Rust code."]
struct Cli {
    #[command(subcommand)]
    tool: Tool,
}

#[derive(Debug, Subcommand)]
enum Tool {
    #[doc = "Download the pinned recording browser into a folder, unless it's already there, and print its path as JSON"]
    InstallEngine {
        #[doc = "The engine folder"]
        folder: PathBuf,
    },
    #[doc = "Print the node catalogue the editor reads, as JSON"]
    Catalog,
    #[doc = "Compile the workflow on standard input and print its film spec, flattened timeline and each bar's frames, as JSON"]
    Film,
    #[doc = "Write the font types the scripts read into scripts/lib/generated"]
    ExportSpec,
    #[doc = "Write every template into examples/ as the app saves a project"]
    WriteExamples,
    #[doc = "Draw a small still of every element through the renderer into the dev cache"]
    RenderElements {
        #[doc = "Draw only the elements whose key contains this"]
        filter: Option<String>,
        #[arg(value_delimiter = ',', default_value = "0.5")]
        #[doc = "Where in each element to draw, as shares of its length, separated by commas"]
        at: Vec<f64>,
    },
}

fn run(tool: Tool) -> Result<(), ToolError> {
    match tool {
        Tool::InstallEngine { folder } => output::with_runtime(engine::install(&folder)),
        Tool::Catalog => catalog::catalog(),
        Tool::Film => catalog::film(),
        Tool::ExportSpec => spec::export(),
        Tool::WriteExamples => examples::write(),
        Tool::RenderElements { filter, at } => {
            output::with_runtime(stills::render(filter.as_deref().unwrap_or_default(), &at))
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse().tool) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("opsis-tools: {error}");
            ExitCode::FAILURE
        }
    }
}
