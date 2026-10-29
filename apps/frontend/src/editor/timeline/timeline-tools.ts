import {
  ArrowLeftRight,
  ArrowRightFromLine,
  Gauge,
  GalleryHorizontal,
  MousePointer2,
  Scissors,
  type LucideIcon,
} from "lucide-react";

export const ZOOM_STEP = 1.5;

export const COMMAND_KEYS = { split: "S", addMarker: "M", fit: "Z" } as const;

export const ZOOM_KEYS = {
  in: { keys: ["=", "+"], label: "=" },
  out: { keys: ["-"], label: "−" },
} as const;

type ToolSpec = { id: string; icon: LucideIcon; label: string; key: string; help: string };

export const TOOLS = [
  {
    id: "select",
    icon: MousePointer2,
    label: "Select",
    key: "V",
    help: "Move clips and drag their edges to trim. Hold Shift to move later clips too.",
  },
  {
    id: "ripple",
    icon: ArrowRightFromLine,
    label: "Ripple",
    key: "B",
    help: "Moves and trims push the clips after them.",
  },
  {
    id: "roll",
    icon: ArrowLeftRight,
    label: "Roll",
    key: "N",
    help: "Drag the cut between two clips. The total length stays the same.",
  },
  {
    id: "slip",
    icon: GalleryHorizontal,
    label: "Slip",
    key: "Y",
    help: "Shows a different part of a clip without changing where it sits.",
  },
  { id: "blade", icon: Scissors, label: "Split", key: "C", help: "Click a clip to cut it in two." },
  {
    id: "rate",
    icon: Gauge,
    label: "Speed",
    key: "R",
    help: "Drag a clip's edge to speed it up or slow it down.",
  },
] as const satisfies readonly ToolSpec[];

export type TimelineTool = (typeof TOOLS)[number]["id"];
