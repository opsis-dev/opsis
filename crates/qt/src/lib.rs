#![allow(
    unsafe_code,
    reason = "opsis-qt is the one crate that drives Qt, through the cxx bridge to its C++ views"
)]

mod bridge;
mod host;

pub use bridge::ffi::{
    CardKind, CardView, Choice, CloseView, DialogKind, EdgeView, EventKind, FieldKind, FieldView,
    GraphView, InspectorView, MenuItemKind, MenuItemView, MenuSectionView, NodeMove, PaletteEntry,
    PortView, ProjectEntry, RowView, SettingsView,
};
pub use host::{
    Host, HostResult, notify, open_path, post_event, qt_version, quit, raise_editor,
    request_dialog, reveal_path, run,
};
