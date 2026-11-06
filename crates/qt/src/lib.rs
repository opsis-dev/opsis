#![allow(
    unsafe_code,
    reason = "opsis-qt is the one crate that drives Qt, through the cxx bridge to its C++ views"
)]

mod bridge;
mod host;

pub use bridge::ffi::{
    CardKind, CardView, Choice, EdgeView, EventKind, FieldKind, FieldView, GraphView,
    InspectorView, NodeMove, PaletteEntry, PortView, ProjectEntry, RowView, SettingsView,
};
pub use host::{Host, HostResult, post_event, run};
