import { clampFrame } from "@opsis/video/common/time";
import { create } from "zustand";

type PreviewPlayback = {
  open: boolean;
  frame: number;
  playing: boolean;
  fps: number;
  durationInFrames: number;
  seek: (frame: number) => void;
  play: () => void;
  pause: () => void;
};

export const usePreviewPlayback = create<PreviewPlayback>()((set, get) => ({
  open: false,
  frame: 0,
  playing: false,
  fps: 30,
  durationInFrames: 1,
  seek: (frame) => {
    const at = clampFrame(frame, get().durationInFrames);
    set({ frame: at });
    if (get().open) void window.opsis.previewSeek(at);
  },
  play: () => {
    if (get().open) void window.opsis.previewPlay();
  },
  pause: () => {
    if (get().open) void window.opsis.previewPause();
  },
}));
