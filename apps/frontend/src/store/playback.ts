import { clamp } from "@opsis/video/common/math";
import { pick } from "@opsis/video/common/objects";
import { clampFrame } from "@opsis/video/common/time";
import { create } from "zustand";
import type { PlaybackCall } from "@/generated/bindings";
import { createForwarder } from "@/lib/forwarder";

type Player = {
  seek: (frame: number) => void;
  play: () => void;
  pause: () => void;
  rate: (rate: number) => void;
};

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
const remote = createForwarder<PlaybackCall>();

export function registerPlayer(controller: Omit<Player, "rate"> | null): void {
  local = controller && {
    seek: (frame) => controller.seek(frame),
    play: () => controller.play(),
    pause: () => controller.pause(),
    rate: (rate) => usePlayback.setState({ rate }),
  };
  usePlayback.setState({ hasPlayer: !!controller });
}

export const forwardPlaybackTo = remote.to;

function callPlayer(player: Player, command: PlaybackCall): void {
  switch (command.call) {
    case "seek":
      return player.seek(command.frame);
    case "play":
      return player.play();
    case "pause":
      return player.pause();
    case "rate":
      return player.rate(command.rate);
  }
}

export function runPlaybackCommand(command: PlaybackCall): void {
  if (local) callPlayer(local, command);
}

const dispatch = (command: PlaybackCall) => {
  if (local) runPlaybackCommand(command);
  else remote.send(command);
};

const MAX_SHUTTLE_RATE = 4;
const SHUTTLE_RATE_STEP = 2;

export const usePlayback = create<PlaybackState>()((set, get) => {
  const setRate = (rate: number) => {
    set({ rate });
    dispatch({ call: "rate", rate });
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
      dispatch({ call: "seek", frame: f });
    },
    play: () => {
      if (get().rate !== 1) setRate(1);
      dispatch({ call: "play" });
    },
    pause: () => dispatch({ call: "pause" }),
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
      if (!playing) dispatch({ call: "play" });
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
