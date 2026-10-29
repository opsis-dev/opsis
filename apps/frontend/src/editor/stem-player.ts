import { clamp } from "@opsis/video/common/math";
import { useEffect, useState } from "react";
import type { StoreApi } from "zustand";
import { dbToGain } from "@/editor/mixer/strip-settings";
import { MAX_PREVIEW_FILTERS, previewTotals, useAudioPreview } from "../store/audio-preview";
import { fileUrl } from "./file-url";

const SMOOTHING_S = 0.02;
const DRIFT_FRAMES = 2;

export type SoundClock = { frame: number; fps: number; playing: boolean; rate?: number };

type Clock = Pick<StoreApi<SoundClock>, "getState" | "subscribe">;

type Chain = {
  context: AudioContext;
  filters: BiquadFilterNode[];
  panner: StereoPannerNode;
  gain: GainNode;
};

function buildChain(audio: HTMLAudioElement): Chain {
  const context = new AudioContext();
  const source = context.createMediaElementSource(audio);
  const filters = Array.from({ length: MAX_PREVIEW_FILTERS }, () => {
    const f = context.createBiquadFilter();
    f.type = "peaking";
    f.gain.value = 0;
    return f;
  });
  const panner = context.createStereoPanner();
  const gain = context.createGain();
  const nodes: AudioNode[] = [source, ...filters, panner, gain, context.destination];
  for (let i = 0; i < nodes.length - 1; i++) nodes[i].connect(nodes[i + 1]);
  return { context, filters, panner, gain };
}

function applyMixerDrags(chain: Chain): void {
  const now = chain.context.currentTime;
  const { gainDb, pan, filters: wanted } = previewTotals(useAudioPreview.getState());
  chain.gain.gain.setTargetAtTime(dbToGain(gainDb), now, SMOOTHING_S);
  chain.panner.pan.setTargetAtTime(clamp(pan, -1, 1), now, SMOOTHING_S);
  chain.filters.forEach((node, i) => {
    const spec = wanted[i];
    if (!spec) {
      node.type = "peaking";
      node.gain.setTargetAtTime(0, now, SMOOTHING_S);
      return;
    }
    node.type = spec.type;
    node.frequency.setTargetAtTime(spec.freq, now, SMOOTHING_S);
    node.Q.setTargetAtTime(spec.q, now, SMOOTHING_S);
    node.gain.setTargetAtTime(spec.gain, now, SMOOTHING_S);
  });
}

export type StemSound = {
  volume: number;
  muted: boolean;
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  start: () => void;
};

function followClock(audio: HTMLAudioElement, clock: Clock): () => void {
  return clock.subscribe((s) => {
    const at = s.frame / s.fps;
    const rate = s.rate ?? 1;
    const playable = s.playing && rate > 0 && !!audio.src;
    if (!playable) {
      if (!audio.paused) audio.pause();
      if (Math.abs(audio.currentTime - at) > DRIFT_FRAMES / s.fps) audio.currentTime = at;
      return;
    }
    audio.playbackRate = rate;
    if (Math.abs(audio.currentTime - at) > DRIFT_FRAMES / s.fps) audio.currentTime = at;
    if (audio.paused) void audio.play().catch(() => undefined);
  });
}

export class StemPlayer {
  private audio: HTMLAudioElement | null = null;
  private chain: Chain | null = null;
  private stops: Array<() => void> = [];
  private stem: string | null = null;
  private volume = 1;
  private muted = false;

  constructor(private readonly clock: Clock) {}

  open(): void {
    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    audio.preload = "auto";
    const chain = buildChain(audio);
    this.audio = audio;
    this.chain = chain;
    applyMixerDrags(chain);
    this.stops = [
      useAudioPreview.subscribe(() => applyMixerDrags(chain)),
      followClock(audio, this.clock),
    ];
    this.play(this.stem);
    this.level(this.volume, this.muted);
  }

  close(): void {
    for (const stop of this.stops) stop();
    this.stops = [];
    this.audio?.pause();
    void this.chain?.context.close();
    this.audio = null;
    this.chain = null;
  }

  play(stem: string | null): void {
    this.stem = stem;
    if (!this.audio) return;
    if (stem) this.audio.src = fileUrl(stem);
    else this.audio.removeAttribute("src");
  }

  level(volume: number, muted: boolean): void {
    this.volume = volume;
    this.muted = muted;
    if (!this.audio) return;
    this.audio.volume = volume;
    this.audio.muted = muted;
  }

  start(): void {
    if (!this.audio?.src) return;
    const { frame, fps } = this.clock.getState();
    void this.chain?.context.resume();
    this.audio.currentTime = frame / fps;
    void this.audio.play().catch(() => undefined);
  }
}

export function useStemSound(clock: Clock, stem: string | null): StemSound {
  const [player] = useState(() => new StemPlayer(clock));
  const [volume, setVolumeState] = useState(1);
  const [muted, setMuted] = useState(false);
  useEffect(() => {
    player.open();
    return () => player.close();
  }, [player]);
  useEffect(() => player.play(stem), [player, stem]);
  useEffect(() => player.level(volume, muted), [player, volume, muted]);
  return {
    volume: muted ? 0 : volume,
    muted,
    setVolume: (next) => {
      setVolumeState(next);
      if (next > 0) setMuted(false);
    },
    toggleMute: () => setMuted((m) => !m),
    start: () => player.start(),
  };
}
