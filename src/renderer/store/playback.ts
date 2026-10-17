import { create } from "zustand";
import { clamp } from "@/shared/math";
import { pick } from "@/shared/pick";
import { frameIn } from "@/shared/timeline/time";
import { createForwarder } from "@/renderer/lib/forwarder";

type PlayerController = {
  seek: (frame: number) => void;
  play: () => void;
  pause: () => void;
  setRate: (rate: number) => void;
};

export type PlaybackCommand =
  | { type: "seek"; frame: number }
  | { type: "play" }
  | { type: "pause" }
  | { type: "rate"; rate: number };

type PlaybackState = {
  frame: number;
  playing: boolean;
  rate: number;
  fps: number;
  durationInFrames: number;
  inPoint: number | null;
  outPoint: number | null;
  hasPlayer: boolean;
  seek: (frame: number) => void;
  play: () => void;
  pause: () => void;
  toggle: () => void;
  step: (frames: number) => void;
  shuttle: (direction: -1 | 0 | 1) => void;
  setIn: (frame: number | null) => void;
  setOut: (frame: number | null) => void;
  clearRange: () => void;
};

let local: PlayerController | null = null;
const remote = createForwarder<PlaybackCommand>();

export function registerPlayer(controller: PlayerController | null): void {
  local = controller;
  usePlayback.setState({ hasPlayer: !!controller });
}

export const forwardPlaybackTo = remote.to;

export function runPlaybackCommand(command: PlaybackCommand): void {
  if (!local) return;
  switch (command.type) {
    case "seek":
      local.seek(command.frame);
      break;
    case "play":
      local.play();
      break;
    case "pause":
      local.pause();
      break;
    case "rate":
      local.setRate(command.rate);
      break;
  }
}

const dispatch = (command: PlaybackCommand) => {
  if (local) runPlaybackCommand(command);
  else remote.send(command);
};

const MAX_SHUTTLE_RATE = 4;

export const usePlayback = create<PlaybackState>()((set, get) => {
  const setRate = (rate: number) => {
    set({ rate });
    dispatch({ type: "rate", rate });
  };
  return {
    frame: 0,
    playing: false,
    rate: 1,
    fps: 30,
    durationInFrames: 1,
    inPoint: null,
    outPoint: null,
    hasPlayer: false,
    seek: (frame) => {
      const f = frameIn(frame, get().durationInFrames);
      set({ frame: f });
      dispatch({ type: "seek", frame: f });
    },
    play: () => {
      if (get().rate !== 1) setRate(1);
      dispatch({ type: "play" });
    },
    pause: () => dispatch({ type: "pause" }),
    toggle: () => (get().playing ? get().pause() : get().play()),
    step: (frames) => {
      get().pause();
      get().seek(get().frame + frames);
    },
    shuttle: (direction) => {
      const { playing, rate } = get();
      if (direction === 0) {
        get().pause();
        setRate(1);
        return;
      }
      const same = playing && Math.sign(rate) === direction;
      const next = same ? clamp(rate * 2, -MAX_SHUTTLE_RATE, MAX_SHUTTLE_RATE) : direction;
      setRate(next);
      if (!playing) dispatch({ type: "play" });
    },
    setIn: (inPoint) =>
      set((s) => ({
        inPoint,
        outPoint:
          inPoint !== null && s.outPoint !== null && s.outPoint <= inPoint ? null : s.outPoint,
      })),
    setOut: (outPoint) =>
      set((s) => ({
        outPoint,
        inPoint:
          outPoint !== null && s.inPoint !== null && s.inPoint >= outPoint ? null : s.inPoint,
      })),
    clearRange: () => set({ inPoint: null, outPoint: null }),
  };
});

export const PLAYBACK_SYNC_KEYS = [
  "frame",
  "playing",
  "rate",
  "fps",
  "durationInFrames",
  "inPoint",
  "outPoint",
] as const;

export const PLAYBACK_LOCAL_KEYS = ["hasPlayer"] as const;

export type PlaybackSnapshot = Pick<PlaybackState, (typeof PLAYBACK_SYNC_KEYS)[number]>;

export const playbackSnapshot = (): PlaybackSnapshot =>
  pick(usePlayback.getState(), PLAYBACK_SYNC_KEYS);
