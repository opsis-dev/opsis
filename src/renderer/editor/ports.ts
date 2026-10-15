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

export const CATEGORY_COLORS: Record<string, string> = {
  Browser: "#38bdf8",
  Capture: "#f97316",
  Camera: "#fb923c",
  Cursor: "#fbbf24",
  "Text & overlays": "#a78bfa",
  Transitions: "#c084fc",
  Backgrounds: "#818cf8",
  Effects: "#e879f9",
  Elements: "#8b5cf6",
  Audio: "#34d399",
  Timeline: "#f472b6",
  Media: "#60a5fa",
  Output: "#f43f5e",
  Utility: "#71717a",
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

export const portLabel = (
  _id: string,
  spec: { type: PortType; label?: string },
  side: "in" | "out",
) => spec.label ?? (spec.type === "session" && side === "in" ? "Previous" : TYPE_LABELS[spec.type]);
