import type { CallbackListener, EventTypes, PlayerRef } from "@remotion/player";
import { useEffect } from "react";
import type { PreviewTarget } from "@/shared/compile/compile";
import { registerPlayer, usePlayback } from "../store/playback";
import { singleSelected, useUiStore } from "../store/ui";

function activeAt(preview: PreviewTarget, frame: number): string[] {
  return Object.entries(preview.spans)
    .filter(([, s]) => frame >= s.from && frame < s.to)
    .map(([id]) => id)
    .sort();
}

function usePlayerEvents(player: PlayerRef | null, preview: PreviewTarget | null): void {
  useEffect(() => {
    if (!player || !preview) return;
    const listen = <E extends EventTypes>(event: E, callback: CallbackListener<E>) => {
      player.addEventListener(event, callback);
      return () => player.removeEventListener(event, callback);
    };
    const update = ({ detail }: { detail: { frame: number } }) => {
      usePlayback.setState({ frame: detail.frame });
      useUiStore.getState().setActive(activeAt(preview, detail.frame));
    };
    const stops = [
      listen("frameupdate", update),
      listen("seeked", update),
      listen("play", () => usePlayback.setState({ playing: true })),
      listen("pause", () => usePlayback.setState({ playing: false })),
    ];
    return () => {
      for (const stop of stops) stop();
    };
  }, [preview, player]);
}

function usePlayerRegistration(player: PlayerRef | null, durationInFrames: number): void {
  useEffect(() => {
    if (!player) return;
    registerPlayer({
      seek: (frame) => player.seekTo(frame),
      play: () => player.play(),
      pause: () => player.pause(),
      setRate: (rate) => usePlayback.setState({ rate }),
    });
    const frame = usePlayback.getState().frame;
    if (frame > 0) player.seekTo(Math.min(frame, durationInFrames - 1));
    return () => {
      registerPlayer(null);
      usePlayback.setState({ playing: false });
    };
  }, [player, durationInFrames]);
}

export function usePlayerBridge(
  player: PlayerRef | null,
  preview: PreviewTarget | null,
  timing: { fps: number; durationInFrames: number },
): void {
  const { fps, durationInFrames } = timing;
  useEffect(() => {
    usePlayback.setState({ fps, durationInFrames });
  }, [fps, durationInFrames]);
  usePlayerRegistration(player, durationInFrames);
  usePlayerEvents(player, preview);
}

export function useSeekToSelection(player: PlayerRef | null, preview: PreviewTarget | null): void {
  const selectedId = useUiStore((s) => (s.pinnedSource ? null : singleSelected(s.selectedIds)));
  useEffect(() => {
    if (!selectedId || !preview) return;
    const span = preview.spans[selectedId];
    if (span) player?.seekTo(span.from);
  }, [selectedId, preview, player]);
}
