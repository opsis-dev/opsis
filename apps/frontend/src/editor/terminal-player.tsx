import { formatDuration } from "@opsis/video/common/format";
import { decodeFrames, frameAt, type TerminalFrames } from "@opsis/video/terminal/frames";
import type { ViewLook } from "@opsis/video/terminal/view";
import { Pause, Play } from "lucide-react";
import { useEffect, useImperativeHandle, useRef, useState, type Ref } from "react";
import { Button } from "@/components/ui/button";
import { Slider } from "@/components/ui/slider";
import { Spinner } from "@/components/ui/spinner";
import { commands, type TerminalTheme } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { TerminalPicture } from "../app/terminal-picture";
import { useWorkflowStore } from "../store/workflow";

export const PLAYER_FPS = 30;
const BLINKS_PER_S = 1;

export const PLAYER_LOOK: ViewLook = {
  fontSize: 15,
  lineHeight: 1.2,
  letterSpacing: 0,
  padding: 12,
  cursor: "program",
};

export type TerminalPlayerHandle = { seek: (seconds: number) => void };

const loaded = new Map<string, Promise<TerminalFrames>>();

export function loadTerminalFrames(
  projectDir: string,
  cast: string,
  fps: number,
  theme: TerminalTheme | null,
): Promise<TerminalFrames> {
  const key = `${projectDir}|${cast}|${fps}|${theme?.name ?? ""}`;
  let frames = loaded.get(key);
  if (!frames) {
    frames = commands
      .terminalFrames(projectDir, cast, fps, theme)
      .then((bytes) => decodeFrames(bytes));
    frames.catch(() => loaded.delete(key));
    loaded.set(key, frames);
  }
  return frames;
}

export function useTerminalFrames(
  cast: string | null,
  fps: number,
  theme: TerminalTheme | null,
): { frames: TerminalFrames | null; error: string | null } {
  const [state, setState] = useState<{
    key: string;
    frames: TerminalFrames | null;
    error: string | null;
  }>({ key: "", frames: null, error: null });
  const projectDir = useWorkflowStore((s) => s.projectDir);
  const key = `${projectDir}|${cast}|${fps}|${theme?.name ?? ""}`;
  useEffect(() => {
    if (!cast || !projectDir) return;
    let alive = true;
    loadTerminalFrames(projectDir, cast, fps, theme).then(
      (frames) => alive && setState({ key, frames, error: null }),
      (error: unknown) => alive && setState({ key, frames: null, error: String(error) }),
    );
    return () => {
      alive = false;
    };
  }, [projectDir, cast, fps, theme, key]);
  return state.key === key ? state : { frames: null, error: null };
}

function lengthOf(frames: TerminalFrames): number {
  return Math.max(0, (frames.frames.length - 1) / frames.fps);
}

function useClock(length: number) {
  const [time, setTime] = useState(0);
  const [playing, setPlaying] = useState(false);
  const started = useRef<{ at: number; from: number } | null>(null);
  const now = useRef(0);
  useEffect(() => {
    now.current = time;
  }, [time]);
  useEffect(() => {
    if (!playing) return;
    let frame = 0;
    const from = now.current >= length ? 0 : now.current;
    started.current = { at: performance.now(), from };
    function tick(at: number) {
      const start = started.current;
      if (!start) return;
      const next = start.from + (at - start.at) / 1000;
      if (next >= length) {
        setTime(length);
        setPlaying(false);
        return;
      }
      setTime(next);
      // oxlint-disable-next-line react/immutability
      frame = requestAnimationFrame(tick);
    }
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      started.current = null;
    };
  }, [playing, length]);
  function seek(seconds: number) {
    const to = Math.min(length, Math.max(0, seconds));
    setTime(to);
    if (started.current) started.current = { at: performance.now(), from: to };
  }
  return { time, playing, setPlaying, seek };
}

export function TerminalPlayer({
  cast,
  theme,
  className,
  onTime,
  handle,
}: {
  cast: string;
  theme: TerminalTheme | null;
  className?: string;
  onTime?: (seconds: number) => void;
  handle?: Ref<TerminalPlayerHandle>;
}) {
  const { frames, error } = useTerminalFrames(cast, PLAYER_FPS, theme);
  const length = frames ? lengthOf(frames) : 0;
  const { time, playing, setPlaying, seek } = useClock(length);
  useImperativeHandle(handle, () => ({ seek }));
  useEffect(() => onTime?.(time), [time, onTime]);
  const frame = frames ? frameAt(frames, time) : undefined;
  return (
    <div className={cn("bg-muted/40 flex flex-col overflow-hidden rounded-md border", className)}>
      <div className="relative aspect-video">
        {frames && frame ? (
          <TerminalPicture
            frames={frames}
            frame={frame}
            look={PLAYER_LOOK}
            cursorOn={Math.floor(time * BLINKS_PER_S * 2) % 2 === 0 || !playing}
          />
        ) : (
          <div className="text-muted-foreground absolute inset-0 grid place-items-center text-xs">
            {error ? "Couldn't show this take" : <Spinner />}
          </div>
        )}
      </div>
      <div className="flex items-center gap-2 px-2 py-1.5">
        <Button
          variant="ghost"
          size="icon-xs"
          aria-label={playing ? "Pause" : "Play"}
          disabled={!frames}
          onClick={() => setPlaying(!playing)}
        >
          {playing ? <Pause /> : <Play />}
        </Button>
        <Slider
          aria-label="Time"
          className="flex-1"
          min={0}
          max={Math.max(length, 0.001)}
          step={1 / PLAYER_FPS}
          value={[time]}
          onValueChange={(value) => seek(Array.isArray(value) ? (value[0] ?? 0) : value)}
        />
        <span className="text-muted-foreground w-24 text-right text-xs tabular-nums">
          {formatDuration(time * 1000)} / {formatDuration(length * 1000)}
        </span>
      </div>
    </div>
  );
}
