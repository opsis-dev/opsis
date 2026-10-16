import type { NodeCategory } from "@/shared/nodes/define";
import type { PortType } from "@/shared/schema/workflow";

export const PORT_COLORS: Record<PortType, string> = {
  session: "#38bdf8",
  recording: "#f97316",
  clip: "#a78bfa",
  scene: "#a78bfa",
  timeline: "#f472b6",
  audio: "#34d399",
  captions: "#facc15",
  image: "#2dd4bf",
  media: "#60a5fa",
  file: "#94a3b8",
};

export const CATEGORY_STYLES: Record<NodeCategory, { color: string; icon: string }> = {
  Browser: { color: "#38bdf8", icon: "globe" },
  Capture: { color: "#f97316", icon: "video" },
  Camera: { color: "#fb923c", icon: "scan-search" },
  Cursor: { color: "#fbbf24", icon: "mouse-pointer-2" },
  "Text & overlays": { color: "#a78bfa", icon: "type" },
  Transitions: { color: "#c084fc", icon: "blend" },
  Backgrounds: { color: "#818cf8", icon: "wallpaper" },
  Effects: { color: "#e879f9", icon: "sparkles" },
  Elements: { color: "#8b5cf6", icon: "shapes" },
  Audio: { color: "#34d399", icon: "music" },
  Timeline: { color: "#f472b6", icon: "layers" },
  Media: { color: "#60a5fa", icon: "file-play" },
  Output: { color: "#f43f5e", icon: "clapperboard" },
  Utility: { color: "#71717a", icon: "sticky-note" },
};

const TYPE_LABELS: Record<PortType, string> = {
  session: "Next",
  recording: "Recording",
  clip: "Clip",
  scene: "Clip",
  timeline: "Clip",
  audio: "Sound",
  captions: "Captions",
  image: "Image",
  media: "File",
  file: "File",
};

export const portLabel = (spec: { type: PortType; label?: string }, side: "in" | "out") =>
  spec.label ?? (spec.type === "session" && side === "in" ? "Previous" : TYPE_LABELS[spec.type]);
