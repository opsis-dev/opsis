import { clamp } from "@opsis/video/common/math";
import { useEffect, useRef, useState } from "react";
import { dbToGain } from "@/core/mix";
import type { PreviewTarget, TimelineSpec, ViewSize } from "@/generated/bindings";
import { attempt } from "@/lib/toast";
import type { StreamFrame } from "../app/opsis";
import { MAX_PREVIEW_FILTERS, previewTotals, useAudioPreview } from "../store/audio-preview";
import { registerPlayer, usePlayback } from "../store/playback";
import { singleSelected, useUiStore } from "../store/ui";
import { useViewOptions } from "../store/view-options";
import { fileUrl } from "./file-url";

const FAST_WIDTH = 960;
const RESIZE_SETTLE_MS = 150;
const SMOOTHING_S = 0.02;
const DRIFT_FRAMES = 2;

function activeAt(preview: PreviewTarget, frame: number): string[] {
  return Object.entries(preview.spans)
    .filter(([, s]) => frame >= s.from && frame < s.to)
    .map(([id]) => id)
    .sort();
}

function viewSize(box: DOMRect, spec: TimelineSpec, fast: boolean): ViewSize {
  const aspect = spec.width / spec.height;
  const width = fast ? FAST_WIDTH : Math.max(1, box.width * devicePixelRatio);
  const height = fast ? FAST_WIDTH / aspect : Math.max(1, box.height * devicePixelRatio);
  const fit = Math.min(width / aspect, height);
  return { width: Math.round(fit * aspect), height: Math.round(fit), dpr: 1 };
}

async function draw(canvas: HTMLCanvasElement, frame: StreamFrame): Promise<void> {
  const bitmap = await createImageBitmap(new Blob([frame.jpeg], { type: "image/jpeg" }));
  if (canvas.width !== bitmap.width) canvas.width = bitmap.width;
  if (canvas.height !== bitmap.height) canvas.height = bitmap.height;
  canvas.getContext("2d")?.drawImage(bitmap, 0, 0);
  bitmap.close();
}

function useOpenPreview(
  projectDir: string | null,
  canvas: HTMLCanvasElement | null,
  size: ViewSize | null,
): boolean {
  const [open, setOpen] = useState(false);
  const latest = useRef(size);
  useEffect(() => {
    latest.current = size;
  }, [size]);
  const sized = size !== null;
  useEffect(() => {
    const opening = latest.current;
    if (!projectDir || !canvas || !sized || !opening) return;
    let gone = false;
    void attempt(async () => {
      await window.opsis.previewOpen(projectDir, opening, (frame) => {
        if (!gone) void draw(canvas, frame);
      });
      if (!gone) setOpen(true);
    }, "Couldn't start the preview");
    return () => {
      gone = true;
      setOpen(false);
      void window.opsis.previewClose();
    };
  }, [projectDir, canvas, sized]);
  return open;
}

function useStageSize(stage: HTMLElement | null, spec: TimelineSpec | null, fast: boolean) {
  const [size, setSize] = useState<ViewSize | null>(null);
  useEffect(() => {
    if (!stage || !spec) return;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const measure = () => setSize(viewSize(stage.getBoundingClientRect(), spec, fast));
    measure();
    const observer = new ResizeObserver(() => {
      clearTimeout(timer);
      timer = setTimeout(measure, RESIZE_SETTLE_MS);
    });
    observer.observe(stage);
    return () => {
      clearTimeout(timer);
      observer.disconnect();
    };
  }, [stage, spec, fast]);
  return size;
}

function useReported(preview: PreviewTarget | null): void {
  useEffect(
    () =>
      window.opsis.onPreviewReported(({ frame, playing }) => {
        usePlayback.setState({ frame, playing });
        if (preview) useUiStore.getState().setActive(activeAt(preview, frame));
      }),
    [preview],
  );
}

function useControls(open: boolean, durationInFrames: number): void {
  const rate = usePlayback((s) => s.rate);
  const inPoint = usePlayback((s) => s.inPoint);
  const outPoint = usePlayback((s) => s.outPoint);
  useEffect(() => {
    if (!open) return;
    registerPlayer({
      seek: (frame) => void window.opsis.previewSeek(frame),
      play: () => void window.opsis.previewPlay(),
      pause: () => void window.opsis.previewPause(),
    });
    const frame = usePlayback.getState().frame;
    if (frame > 0) void window.opsis.previewSeek(Math.min(frame, durationInFrames - 1));
    return () => {
      registerPlayer(null);
      usePlayback.setState({ playing: false });
    };
  }, [open, durationInFrames]);
  useEffect(() => {
    if (open) void window.opsis.previewRate(rate);
  }, [open, rate]);
  useEffect(() => {
    if (!open) return;
    const last = durationInFrames - 1;
    const range =
      (inPoint === null && outPoint === null) || durationInFrames < 2
        ? {}
        : {
            inFrame: clamp(inPoint ?? 0, 0, last - 1),
            outFrame: clamp(outPoint ?? last, (inPoint ?? 0) + 1, last),
          };
    void window.opsis.previewRange(range);
  }, [open, inPoint, outPoint, durationInFrames]);
}

function useSeekToSelection(open: boolean, preview: PreviewTarget | null): void {
  const selectedId = useUiStore((s) => (s.pinnedSource ? null : singleSelected(s.selectedIds)));
  useEffect(() => {
    if (!open || !selectedId || !preview) return;
    const span = preview.spans[selectedId];
    if (span) usePlayback.getState().seek(span.from);
  }, [open, selectedId, preview]);
}

export function useStreamedPreview({
  projectDir,
  preview,
  spec,
  stage,
  canvas,
}: {
  projectDir: string | null;
  preview: PreviewTarget | null;
  spec: TimelineSpec | null;
  stage: HTMLElement | null;
  canvas: HTMLCanvasElement | null;
}): void {
  const fast = useViewOptions((s) => s.previewQuality) === "fast";
  const size = useStageSize(stage, spec, fast);
  const open = useOpenPreview(projectDir, canvas, size);
  const durationInFrames = Math.max(1, Math.round(spec?.durationInFrames ?? 1));
  useEffect(() => {
    usePlayback.setState({ fps: spec?.fps ?? 30, durationInFrames });
  }, [spec?.fps, durationInFrames]);
  useEffect(() => {
    if (open && spec) void window.opsis.previewLoad(spec);
  }, [open, spec]);
  useEffect(() => {
    if (open && size) void window.opsis.previewResize(size);
  }, [open, size]);
  useReported(preview);
  useControls(open, durationInFrames);
  useSeekToSelection(open, preview);
}

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

export type PreviewSound = {
  volume: number;
  muted: boolean;
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  start: () => void;
};

function followFrames(audio: HTMLAudioElement): () => void {
  return usePlayback.subscribe((s) => {
    const at = s.frame / s.fps;
    const playable = s.playing && s.rate > 0 && !!audio.src;
    if (!playable) {
      if (!audio.paused) audio.pause();
      if (Math.abs(audio.currentTime - at) > DRIFT_FRAMES / s.fps) audio.currentTime = at;
      return;
    }
    audio.playbackRate = s.rate;
    if (Math.abs(audio.currentTime - at) > DRIFT_FRAMES / s.fps) audio.currentTime = at;
    if (audio.paused) void audio.play().catch(() => undefined);
  });
}

class StemPlayer {
  private audio: HTMLAudioElement | null = null;
  private chain: Chain | null = null;
  private stops: Array<() => void> = [];
  private stem: string | null = null;
  private volume = 1;
  private muted = false;

  open(): void {
    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    audio.preload = "auto";
    const chain = buildChain(audio);
    this.audio = audio;
    this.chain = chain;
    applyMixerDrags(chain);
    this.stops = [useAudioPreview.subscribe(() => applyMixerDrags(chain)), followFrames(audio)];
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
    const { frame, fps } = usePlayback.getState();
    this.audio.currentTime = frame / fps;
    void this.audio.play().catch(() => undefined);
  }
}

export function usePreviewSound(stem: string | null): PreviewSound {
  const [player] = useState(() => new StemPlayer());
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
