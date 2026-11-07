#![allow(
    unsafe_code,
    reason = "opsis-qt is the one crate that drives Qt, through the cxx bridge to its C++ views"
)]

#[expect(
    clippy::unnecessary_box_returns,
    reason = "cxx hands an opaque Rust type to C++ only inside a Box"
)]
mod bridge;
mod frame;
mod host;

pub use bridge::ffi::{
    ArtifactView, BarView, BoundsView, CardKind, CardView, CategoryView, Choice, CloseView,
    ColorView, DialogKind, EdgeView, EventKind, FieldKind, FieldView, FilmTile, FilmTiles,
    GradientInfo, GradientStopView, GraphView, InspectorView, IssueView, KeyLineView, KeyPointView,
    LoudnessView, MarkDrawnView, MarkPointsView, MenuItemKind, MenuItemView, MenuSectionView,
    MeterView, MixerStripView, MixerView, MotionInfo, NodeMove, PaletteEntry, PlayerView, PortView,
    PoseView, PresetView, ProjectEntry, ProjectView, RemovalView, ReplayActionView, ReplayView,
    RowView, RulerStep, RunLogView, RunPanelView, RunRowView, ScreenArea, SettingsView, ShapeView,
    SnapView, TemplateView, TextToken, TimelineAudioView, TimelineBarView, TimelineCameraKeyView,
    TimelineMarkView, TimelineMarkerView, TimelineTransitionView, TimelineView, TokenKind,
    TraceActionView, TraceConsoleView, TraceErrorView, TraceFrameView, TraceRequestView, TraceView,
    VariableView, WindowPlacement,
};
pub use frame::FrameRef;
pub use host::{
    Host, HostResult, notify, open_path, post_event, qt_version, quit, raise_editor,
    request_dialog, reveal_path, run, wake,
};
