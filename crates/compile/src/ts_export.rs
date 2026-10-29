use opsis_types::actions::ActionLog;
use opsis_types::timeline::TimelineSpec;
use opsis_types::ts_export::{boundary_types, render_plain};
use opsis_types::values::CaptionsValue;
use specta::Types;

use crate::catalog::{CatalogEntry, CatalogNode};
use crate::check::CheckResult;
use crate::compile::{CompileResult, PreviewTarget};
use crate::elements::props::ElementSettings;
use crate::graph::{ConnectionProblem, GraphIssue};
use crate::mix::plan::MixPlan;
use crate::timeline::chapters::FilmChapter;
use crate::timeline::flatten::FlatTimeline;

pub fn frontend_types() -> Types {
    boundary_types()
        .register::<CompileResult>()
        .register::<PreviewTarget>()
        .register::<GraphIssue>()
        .register::<ConnectionProblem>()
        .register::<FlatTimeline>()
        .register::<MixPlan>()
        .register::<FilmChapter>()
        .register::<CatalogNode>()
        .register::<CatalogEntry>()
        .register::<CheckResult>()
}

pub const SPEC_TYPES_FILE: &str = "packages/video/src/generated/spec.ts";

pub fn spec_types() -> String {
    render_plain(
        &Types::default()
            .register::<TimelineSpec>()
            .register::<CaptionsValue>()
            .register::<ActionLog>()
            .register::<ElementSettings>()
            .register::<crate::elements::introspect::FieldWidget>()
            .register::<opsis_types::looks::NodeLook>()
            .register::<crate::elements::text::text_3d::PreparedTypeface>(),
    )
}
