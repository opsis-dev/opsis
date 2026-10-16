import { create } from "zustand";

export type PlayerController = {
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
  report: (
    patch: Partial<Pick<PlaybackState, "frame" | "playing" | "fps" | "durationInFrames" | "rate">>,
  ) => void;
  seek: (frame: number) => void;
  play: () => void;
  pause: () => void;
  toggle: () => void;
  step: (frames: number) => void;
  shuttle: (direction: -1 | 0 | 1) => void;
  setIn: (frame: number | null) => void;
  setOut: (frame: number | null) => void;
};

let local: PlayerController | null = null;
let remote: ((command: PlaybackCommand) => void) | null = null;

export function registerPlayer(controller: PlayerController | null): void {
  local = controller;
  usePlayback.setState({ hasPlayer: !!controller });
}

export function forwardPlaybackTo(send: ((command: PlaybackCommand) => void) | null): void {
  remote = send;
}

export function runPlaybackCommand(command: PlaybackCommand): void {
  if (!local) return;
  if (command.type === "seek") local.seek(command.frame);
  else if (command.type === "play") local.play();
  else if (command.type === "pause") local.pause();
  else local.setRate(command.rate);
}

const dispatch = (command: PlaybackCommand) => {
  if (local) runPlaybackCommand(command);
  else remote?.(command);
};

const clamp = (frame: number, total: number) =>
  Math.max(0, Math.min(Math.max(0, total - 1), Math.round(frame)));

export const usePlayback = create<PlaybackState>()((set, get) => ({
  frame: 0,
  playing: false,
  rate: 1,
  fps: 30,
  durationInFrames: 1,
  inPoint: null,
  outPoint: null,
  hasPlayer: false,
  report: (patch) => set(patch),
  seek: (frame) => {
    const f = clamp(frame, get().durationInFrames);
    set({ frame: f });
    dispatch({ type: "seek", frame: f });
  },
  play: () => {
    if (get().rate !== 1) {
      set({ rate: 1 });
      dispatch({ type: "rate", rate: 1 });
    }
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
      dispatch({ type: "pause" });
      set({ rate: 1 });
      dispatch({ type: "rate", rate: 1 });
      return;
    }
    const same = playing && Math.sign(rate) === direction;
    const next = same ? Math.max(-4, Math.min(4, rate * 2)) : direction;
    set({ rate: next });
    dispatch({ type: "rate", rate: next });
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
      inPoint: outPoint !== null && s.inPoint !== null && s.inPoint >= outPoint ? null : s.inPoint,
    })),
}));
