import { atLeastOneFrame } from "@opsis/video/common/time";
import { useEffect, useState } from "react";
import type { FilmTarget } from "@/generated/bindings";
import { registerPlayer, usePlayback } from "@/store/playback";
import { singleSelected, useUiStore } from "@/store/ui";
import { activeAt, loopOf, wrapInLoop, type Loop } from "@/timeline/playhead";
import { useFilm } from "../film";
import { StemPlayer } from "../stem-player";
import { useCompiledContext } from "../use-compiled";

function currentLoop(): Loop {
  const { inPoint, outPoint, durationInFrames } = usePlayback.getState();
  return loopOf(inPoint, outPoint, durationInFrames);
}

class PlayheadClock {
  private position = 0;
  private last = 0;
  private request = 0;

  constructor(private readonly sound: StemPlayer) {}

  seek(frame: number): void {
    this.position = frame;
  }

  play(): void {
    if (this.request) return;
    this.position = usePlayback.getState().frame;
    this.last = performance.now();
    usePlayback.setState({ playing: true });
    this.sound.start();
    this.request = requestAnimationFrame(this.tick);
  }

  pause(): void {
    cancelAnimationFrame(this.request);
    this.request = 0;
    usePlayback.setState({ playing: false });
  }

  private readonly tick = (now: number): void => {
    const { fps, rate } = usePlayback.getState();
    const moved = this.position + ((now - this.last) / 1000) * fps * rate;
    this.last = now;
    this.position = wrapInLoop(moved, currentLoop());
    usePlayback.setState({ frame: Math.floor(this.position) });
    this.request = requestAnimationFrame(this.tick);
  };
}

function useFilmLength(fps: number, durationInFrames: number): void {
  useEffect(() => {
    usePlayback.setState({ fps, durationInFrames });
    const playback = usePlayback.getState();
    if (playback.frame >= durationInFrames) playback.seek(durationInFrames - 1);
  }, [fps, durationInFrames]);
}

function useActiveNodes(film: FilmTarget | null): void {
  useEffect(() => {
    const setActive = useUiStore.getState().setActive;
    if (!film) {
      setActive([]);
      return;
    }
    setActive(activeAt(film.spans, usePlayback.getState().frame));
    return usePlayback.subscribe((s, last) => {
      if (s.frame !== last.frame) setActive(activeAt(film.spans, s.frame));
    });
  }, [film]);
}

function useSeekToSelection(film: FilmTarget | null): void {
  const selectedId = useUiStore((s) => (s.pinnedSource ? null : singleSelected(s.selectedIds)));
  useEffect(() => {
    if (!selectedId || !film) return;
    const span = film.spans[selectedId];
    if (span) usePlayback.getState().seek(span.from);
  }, [selectedId, film]);
}

export function TimelineClock(): null {
  const film = useFilm();
  const { film: picked } = useCompiledContext();
  const [sound] = useState(() => new StemPlayer(usePlayback));
  const [clock] = useState(() => new PlayheadClock(sound));
  useFilmLength(film.spec?.fps ?? 30, atLeastOneFrame(film.spec?.durationInFrames ?? 1));
  useEffect(() => {
    sound.open();
    registerPlayer(clock);
    return () => {
      clock.pause();
      registerPlayer(null);
      sound.close();
    };
  }, [sound, clock]);
  useEffect(() => sound.play(film.stem), [sound, film.stem]);
  useActiveNodes(picked);
  useSeekToSelection(picked);
  return null;
}
