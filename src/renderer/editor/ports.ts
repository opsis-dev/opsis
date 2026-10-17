import type { NodeCategory } from "@/shared/nodes/define";
import { getNodeDef } from "@/shared/nodes/registry";
import type { PortType } from "@/shared/schema/workflow";

const NEUTRAL_COLOR = "#71717a";

const CLIP_PORT = { color: "#a78bfa", label: "Clip" };

const PORT_TYPES: Record<PortType, { color: string; label: string }> = {
  session: { color: "#38bdf8", label: "Next" },
  recording: { color: "#f97316", label: "Recording" },
  clip: CLIP_PORT,
  scene: CLIP_PORT,
  timeline: { color: "#f472b6", label: "Clip" },
  audio: { color: "#34d399", label: "Sound" },
  captions: { color: "#facc15", label: "Captions" },
  image: { color: "#2dd4bf", label: "Image" },
  media: { color: "#60a5fa", label: "File" },
  file: { color: "#94a3b8", label: "File" },
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
  Utility: { color: NEUTRAL_COLOR, icon: "sticky-note" },
};

export const portTypeColor = (type: PortType | undefined): string =>
  type ? PORT_TYPES[type].color : NEUTRAL_COLOR;

export function categoryColor(nodeType: string): string {
  const category = getNodeDef(nodeType)?.category;
  return category ? CATEGORY_STYLES[category].color : NEUTRAL_COLOR;
}

export const portLabel = (spec: { type: PortType; label?: string }, side: "in" | "out") =>
  spec.label ??
  (spec.type === "session" && side === "in" ? "Previous" : PORT_TYPES[spec.type].label);
