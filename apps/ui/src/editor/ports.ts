import type {
  Category as NodeCategory,
  Icon as NodeIconName,
  PortType,
} from "@/generated/bindings";
import { getNodeDef } from "@/store/catalog";

const NEUTRAL_COLOR = "#71717a";

export const CATEGORY_STYLES: Record<NodeCategory, { color: string; icon: NodeIconName }> = {
  Browser: { color: "#38bdf8", icon: "globe" },
  Capture: { color: "#f97316", icon: "video" },
  Camera: { color: "#fb923c", icon: "scan-search" },
  Cursor: { color: "#fbbf24", icon: "mouse-pointer-2" },
  "Text & overlays": { color: "#a78bfa", icon: "type" },
  Transitions: { color: "#c084fc", icon: "blend" },
  Backgrounds: { color: "#818cf8", icon: "wallpaper" },
  Effects: { color: "#e879f9", icon: "sparkles" },
  Grade: { color: "#fb7185", icon: "palette" },
  Elements: { color: "#8b5cf6", icon: "shapes" },
  Audio: { color: "#34d399", icon: "music" },
  Timeline: { color: "#f472b6", icon: "layers" },
  Media: { color: "#60a5fa", icon: "file-play" },
  Image: { color: "#2dd4bf", icon: "image" },
  Output: { color: "#f43f5e", icon: "clapperboard" },
  Utility: { color: NEUTRAL_COLOR, icon: "sticky-note" },
};

const PORT_COLORS: Record<PortType, string> = {
  "browser-session": CATEGORY_STYLES.Browser.color,
  recording: CATEGORY_STYLES.Capture.color,
  "recording-style": CATEGORY_STYLES.Cursor.color,
  grade: CATEGORY_STYLES.Grade.color,
  clip: CATEGORY_STYLES["Text & overlays"].color,
  audio: CATEGORY_STYLES.Audio.color,
  captions: "#facc15",
  image: "#2dd4bf",
  media: CATEGORY_STYLES.Media.color,
};

export function portTypeColor(type: PortType | undefined): string {
  return type ? PORT_COLORS[type] : NEUTRAL_COLOR;
}

export function categoryColor(nodeType: string): string {
  const category = getNodeDef(nodeType)?.category;
  return category ? CATEGORY_STYLES[category].color : NEUTRAL_COLOR;
}
