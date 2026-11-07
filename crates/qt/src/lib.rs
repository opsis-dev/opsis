#![allow(
    unsafe_code,
    reason = "opsis-qt is the one crate that drives Qt, through the cxx bridge to its C++ views"
)]

mod bridge;
mod host;

pub use bridge::ffi::{
    ArtifactView, BarView, BoundsView, CardKind, CardView, CategoryView, Choice, CloseView,
    ColorView, DialogKind, EdgeView, EventKind, FieldKind, FieldView, FilmTile, FilmTiles,
    GradientInfo, GradientStopView, GraphView, InspectorView, IssueView, KeyLineView, KeyPointView,
    LoudnessView, MarkDrawnView, MarkPointsView, MenuItemKind, MenuItemView, MenuSectionView,
    MeterView, MixerStripView, MixerView, MotionInfo, NodeMove, PaletteEntry, PlayerFrame,
    PlayerView, PortView, PoseView, PresetView, ProjectEntry, ProjectView, RemovalView,
    ReplayActionView, ReplayView, RowView, RulerStep, RunLogView, RunPanelView, RunRowView,
    ScreenArea, SettingsView, ShapeView, SnapView, TemplateView, TextToken, TimelineAudioView,
    TimelineBarView, TimelineCameraKeyView, TimelineMarkView, TimelineMarkerView,
    TimelineTransitionView, TimelineView, TokenKind, TraceActionView, TraceConsoleView,
    TraceErrorView, TraceFrameView, TraceRequestView, TraceView, VariableView, WindowPlacement,
};
pub use host::{
    Host, HostResult, notify, open_path, post_event, qt_version, quit, raise_editor,
    request_dialog, reveal_path, run,
};
