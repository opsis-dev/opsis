import { create } from "zustand";
import { createForwarder } from "@/renderer/lib/forwarder";
import { clamp } from "@/shared/math";
import { pick } from "@/shared/objects";
import { clampFrame } from "@/shared/timeline/time";

type PlayerCalls = { seek: [frame: number]; play: []; pause: []; rate: [rate: number] };

type Player = { [K in keyof PlayerCalls]: (...args: PlayerCalls[K]) => void };

export type PlaybackCommand = {
  [K in keyof PlayerCalls]: { call: K; args: PlayerCalls[K] };
}[keyof PlayerCalls];

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

let local: Player | null = null;
const remote = createForwarder<PlaybackCommand>();

export function registerPlayer(controller: Omit<Player, "rate"> | null): void {
  local = controller && { ...controller, rate: (rate) => usePlayback.setState({ rate }) };
  usePlayback.setState({ hasPlayer: !!controller });
}

export const forwardPlaybackTo = remote.to;

function callPlayer<K extends keyof PlayerCalls>(player: Player, call: K, args: PlayerCalls[K]) {
  player[call](...args);
}

export function runPlaybackCommand({ call, args }: PlaybackCommand): void {
  if (local) callPlayer(local, call, args);
}

const dispatch = (command: PlaybackCommand) => {
  if (local) runPlaybackCommand(command);
  else remote.send(command);
};

const MAX_SHUTTLE_RATE = 4;
const SHUTTLE_RATE_STEP = 2;

export const usePlayback = create<PlaybackState>()((set, get) => {
  const setRate = (rate: number) => {
    set({ rate });
    dispatch({ call: "rate", args: [rate] });
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
      const f = clampFrame(frame, get().durationInFrames);
      set({ frame: f });
      dispatch({ call: "seek", args: [f] });
    },
    play: () => {
      if (get().rate !== 1) setRate(1);
      dispatch({ call: "play", args: [] });
    },
    pause: () => dispatch({ call: "pause", args: [] }),
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
      const next = same
        ? clamp(rate * SHUTTLE_RATE_STEP, -MAX_SHUTTLE_RATE, MAX_SHUTTLE_RATE)
        : direction;
      setRate(next);
      if (!playing) dispatch({ call: "play", args: [] });
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

const PLAYBACK_SYNC_KEYS = [
  "frame",
  "playing",
  "rate",
  "fps",
  "durationInFrames",
  "inPoint",
  "outPoint",
] as const;

export type PlaybackSnapshot = Pick<PlaybackState, (typeof PLAYBACK_SYNC_KEYS)[number]>;

export const playbackSnapshot = (): PlaybackSnapshot =>
  pick(usePlayback.getState(), PLAYBACK_SYNC_KEYS);
