#![allow(
    unsafe_code,
    reason = "opsis-qt is the one crate that drives Qt, through the cxx bridge to its C++ views"
)]

mod bridge;
mod host;

pub use bridge::ffi::{
    BarView, BoundsView, CardKind, CardView, CategoryView, Choice, CloseView, ColorView,
    DialogKind, EdgeView, EventKind, FieldKind, FieldView, FileView, GradientInfo,
    GradientStopView, GraphView, InspectorView, IssueView, MarkDrawnView, MarkPointsView,
    MenuItemKind, MenuItemView, MenuSectionView, MotionInfo, NodeMove, PaletteEntry, PortView,
    PoseView, PresetView, ProjectEntry, ProjectView, RemovalView, RowView, RulerStep, ScreenArea,
    SettingsView, ShapeView, SnapView, TemplateView, TextToken, TimelineAudioView, TimelineBarView,
    TimelineCameraKeyView, TimelineMarkView, TimelineMarkerView, TimelineTransitionView,
    TimelineView, TokenKind, VariableView, WindowPlacement,
};
pub use host::{
    Host, HostResult, notify, open_path, post_event, qt_version, quit, raise_editor,
    request_dialog, reveal_path, run,
};
