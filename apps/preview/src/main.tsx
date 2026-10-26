import type { TimelineSpec } from "@opsis/video/generated/spec";
import { DemoTimeline } from "@opsis/video/player";
import { Player, type CallbackListener, type PlayerRef } from "@remotion/player";
import { StrictMode, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import "./preview.css";

type Binding = (payload: string) => void;

type PreviewApi = {
  load: (spec: TimelineSpec) => void;
  play: () => void;
  pause: () => void;
  seek: (frame: number) => void;
  rate: (rate: number) => void;
  range: (inFrame: number | null, outFrame: number | null) => void;
};

type PlayRange = { inFrame: number | null; outFrame: number | null };

declare global {
  interface Window {
    opsisPreview?: PreviewApi;
    opsisFrame?: Binding;
    opsisState?: Binding;
  }
}

const atLeastOneFrame = (frames: number) => Math.max(1, Math.round(frames));

function Preview() {
  const [spec, setSpec] = useState<TimelineSpec | null>(null);
  const [player, setPlayer] = useState<PlayerRef | null>(null);
  const [rate, setRate] = useState(1);
  const [range, setRange] = useState<PlayRange>({ inFrame: null, outFrame: null });

  useEffect(() => {
    window.opsisPreview = {
      load: setSpec,
      play: () => player?.play(),
      pause: () => player?.pause(),
      seek: (frame) => player?.seekTo(frame),
      rate: setRate,
      range: (inFrame, outFrame) => setRange({ inFrame, outFrame }),
    };
    window.opsisState?.(player ? "ready" : "waiting");
  }, [player]);

  useEffect(() => {
    if (!player) return;
    const frame: CallbackListener<"frameupdate"> = (e) =>
      window.opsisFrame?.(String(e.detail.frame));
    const seeked: CallbackListener<"seeked"> = (e) => window.opsisFrame?.(String(e.detail.frame));
    const playing = () => window.opsisState?.("playing");
    const paused = () => window.opsisState?.("paused");
    player.addEventListener("frameupdate", frame);
    player.addEventListener("seeked", seeked);
    player.addEventListener("play", playing);
    player.addEventListener("pause", paused);
    player.addEventListener("ended", paused);
    return () => {
      player.removeEventListener("frameupdate", frame);
      player.removeEventListener("seeked", seeked);
      player.removeEventListener("play", playing);
      player.removeEventListener("pause", paused);
      player.removeEventListener("ended", paused);
    };
  }, [player]);

  if (!spec) return null;
  return (
    <Player
      ref={setPlayer}
      component={DemoTimeline}
      inputProps={spec}
      durationInFrames={atLeastOneFrame(spec.durationInFrames)}
      fps={spec.fps}
      compositionWidth={spec.width}
      compositionHeight={spec.height}
      playbackRate={rate}
      inFrame={range.inFrame}
      outFrame={range.outFrame}
      controls={false}
      loop
      clickToPlay
      initiallyMuted
      style={{ width: "100%", height: "100%" }}
      acknowledgeRemotionLicense
    />
  );
}

const root = document.getElementById("root");
if (root)
  createRoot(root).render(
    <StrictMode>
      <Preview />
    </StrictMode>,
  );
