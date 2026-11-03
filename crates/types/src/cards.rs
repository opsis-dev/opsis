use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Icon {
    AppWindow,
    ArrowRightLeft,
    AudioLines,
    Blend,
    Bookmark,
    Captions,
    Check,
    CircleCheck,
    CircleDot,
    Clapperboard,
    Clock,
    Code,
    CodeXml,
    #[serde(rename = "columns-2")]
    Columns2,
    Combine,
    Command,
    Crop,
    Crosshair,
    Eye,
    EyeOff,
    FastForward,
    File,
    FilePlay,
    FileVideo,
    Film,
    Flashlight,
    Gauge,
    GitBranch,
    Globe,
    Grab,
    HardDrive,
    Hourglass,
    Image,
    ImagePlay,
    Info,
    KeyRound,
    Keyboard,
    Languages,
    Laptop,
    Layers,
    Link,
    List,
    ListChecks,
    ListVideo,
    LocateFixed,
    Map,
    Maximize,
    MessageSquare,
    Monitor,
    Moon,
    MousePointer,
    #[serde(rename = "mouse-pointer-2")]
    MousePointer2,
    MousePointerClick,
    Move,
    MoveUpRight,
    Music,
    Palette,
    Presentation,
    Ratio,
    RefreshCw,
    Repeat,
    Reply,
    Rocket,
    Ruler,
    Scan,
    ScanSearch,
    Scissors,
    Scroll,
    ScrollText,
    Server,
    #[serde(rename = "settings-2")]
    Settings2,
    Shapes,
    ShieldAlert,
    ShieldCheck,
    Smartphone,
    Sparkles,
    Spline,
    SquareDashed,
    SquareTerminal,
    StickyNote,
    Sun,
    Timer,
    TimerOff,
    TriangleAlert,
    Type,
    Upload,
    Video,
    #[serde(rename = "volume-2")]
    Volume2,
    VolumeX,
    Wallpaper,
    Wifi,
    Wind,
    ZoomIn,
    ZoomOut,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type, schemars::JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Tone {
    Muted,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, schemars::JsonSchema)]
pub struct CardRow {
    pub icon: Icon,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tone: Option<Tone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swatches: Option<Vec<String>>,
}
