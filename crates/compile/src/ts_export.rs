use opsis_types::ts_export::{boundary_types, render};

use crate::catalog::{CatalogEntry, CatalogNode};
use crate::check::CheckResult;
use crate::compile::{CompileResult, PreviewTarget};
use crate::graph::{ConnectionProblem, GraphIssue};
use crate::mix::plan::MixPlan;
use crate::timeline::chapters::FilmChapter;
use crate::timeline::flatten::FlatTimeline;

pub const BINDINGS_FILE: &str = "apps/ui/src/generated/bindings.ts";

pub fn bindings() -> String {
    let types = boundary_types()
        .register::<CompileResult>()
        .register::<PreviewTarget>()
        .register::<GraphIssue>()
        .register::<ConnectionProblem>()
        .register::<FlatTimeline>()
        .register::<MixPlan>()
        .register::<FilmChapter>()
        .register::<CatalogNode>()
        .register::<CatalogEntry>()
        .register::<CheckResult>();
    render(&types)
}
