import { isBoolean } from "@opsis/core/guards";
import { clamp } from "@opsis/core/math";
import type { AudioEnvelopeKey, VisualEnvelopeKey } from "@opsis/core/schema/timeline";
import { formatTime } from "@opsis/core/timeline/time";
import { create } from "zustand";
import { usePlayback } from "../../store/playback";
import { readPref, writePref } from "../../store/view-prefs";
import type { TimelineTool } from "./timeline-tools";

type RulerMode = "seconds" | "timecode";

type Prefs = { rulerMode: RulerMode; snapping: boolean };

const storedTimelinePrefs = (): Prefs => ({
  rulerMode: readPref<RulerMode>(
    "rulerMode",
    "seconds",
    (v) => v === "seconds" || v === "timecode",
  ),
  snapping: readPref("snapping", true, isBoolean),
});

type TimelineState = Prefs & {
  pxPerFrame: number;
  fit: boolean;
  tool: TimelineTool;
  videoEnvelope: VisualEnvelopeKey | null;
  audioEnvelope: AudioEnvelopeKey;
  selectedMarker: string | null;
  setZoom: (pxPerFrame: number) => void;
  fitToView: () => void;
  setTool: (tool: TimelineTool) => void;
  setSnapping: (on: boolean) => void;
  setRulerMode: (mode: RulerMode) => void;
  setVideoEnvelope: (key: VisualEnvelopeKey | null) => void;
  setAudioEnvelope: (key: AudioEnvelopeKey) => void;
  selectMarker: (id: string | null) => void;
  resetForProject: () => void;
};

const MIN_PX_PER_FRAME = 0.01;
const MAX_PX_PER_FRAME = 24;

export const clampZoom = (pxPerFrame: number): number =>
  clamp(pxPerFrame, MIN_PX_PER_FRAME, MAX_PX_PER_FRAME);

export const useTimeline = create<TimelineState>()((set) => ({
  ...storedTimelinePrefs(),
  pxPerFrame: 2,
  fit: true,
  tool: "select",
  videoEnvelope: null,
  audioEnvelope: "volume",
  selectedMarker: null,
  setZoom: (pxPerFrame) => set({ pxPerFrame: clampZoom(pxPerFrame), fit: false }),
  fitToView: () => set({ fit: true }),
  setTool: (tool) => set({ tool }),
  setSnapping: (snapping) => {
    set({ snapping });
    writePref("snapping", snapping);
  },
  setRulerMode: (rulerMode) => {
    set({ rulerMode });
    writePref("rulerMode", rulerMode);
  },
  setVideoEnvelope: (videoEnvelope) => set({ videoEnvelope }),
  setAudioEnvelope: (audioEnvelope) => set({ audioEnvelope }),
  selectMarker: (selectedMarker) => set({ selectedMarker }),
  resetForProject: () => set({ ...storedTimelinePrefs(), fit: true, selectedMarker: null }),
}));

export function useFormatTime(fps: number): (frame: number) => string {
  const rulerMode = useTimeline((s) => s.rulerMode);
  return (frame) => formatTime(frame, fps, rulerMode);
}

export function useLoopRange(duration: number): { from: number; to: number } | null {
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  if (inPoint === null && outPoint === null) return null;
  return { from: inPoint ?? 0, to: outPoint ?? duration };
}
