import type { PlayerRef } from "@remotion/player";
import { Maximize2, Minimize2, Pause, Play, Volume2, VolumeX } from "lucide-react";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
  type ReactNode,
} from "react";
import { Button } from "@/renderer/components/ui/button";
import { Kbd } from "@/renderer/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/renderer/components/ui/tooltip";
import { cn } from "@/renderer/lib/utils";
import { SingleSlider } from "../app/single-slider";
import { usePlayback } from "../store/playback";
import { useFormatTime } from "./timeline/timeline-state";

type Fullscreen = { active: boolean; toggle: () => void };

function togglePlayback(player: PlayerRef | null, gesture: React.SyntheticEvent) {
  const playback = usePlayback.getState();
  if (!player || playback.playing) {
    playback.toggle();
    return;
  }
  if (playback.rate !== 1) usePlayback.setState({ rate: 1 });
  player.play(gesture);
}

function useFullscreen(target: HTMLElement | null): Fullscreen {
  const [active, setActive] = useState(false);
  useEffect(() => {
    const update = () => setActive(!!target && document.fullscreenElement === target);
    document.addEventListener("fullscreenchange", update);
    return () => document.removeEventListener("fullscreenchange", update);
  }, [target]);
  return {
    active,
    toggle: () => {
      if (document.fullscreenElement) void document.exitFullscreen();
      else void target?.requestFullscreen();
    },
  };
}

function usePlayerVolume(player: PlayerRef | null) {
  const subscribe = useCallback(
    (notify: () => void) => {
      if (!player) return () => undefined;
      player.addEventListener("volumechange", notify);
      player.addEventListener("mutechange", notify);
      return () => {
        player.removeEventListener("volumechange", notify);
        player.removeEventListener("mutechange", notify);
      };
    },
    [player],
  );
  const volume = useSyncExternalStore(subscribe, () => player?.getVolume() ?? 1);
  const muted = useSyncExternalStore(subscribe, () => player?.isMuted() ?? false);
  return {
    volume: muted ? 0 : volume,
    muted,
    setVolume: (next: number) => {
      player?.setVolume(next);
      if (next > 0 && muted) player?.unmute();
    },
    toggleMute: () => (muted ? player?.unmute() : player?.mute()),
  };
}

function ControlButton({
  label,
  shortcut,
  container,
  className,
  onClick,
  children,
}: {
  label: string;
  shortcut?: string;
  container: HTMLElement | undefined;
  className?: string;
  onClick: (e: React.MouseEvent) => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon"
            aria-label={label}
            className={cn("size-8", className)}
            onClick={onClick}
          />
        }
      >
        {children}
      </TooltipTrigger>
      <TooltipContent container={container}>
        {label}
        {shortcut ? <Kbd>{shortcut}</Kbd> : null}
      </TooltipContent>
    </Tooltip>
  );
}

function SeekBar({ duration }: { duration: number }) {
  const frame = usePlayback((s) => s.frame);
  const resumeAfterScrub = useRef(false);
  const seek = (next: number) => {
    const playback = usePlayback.getState();
    if (playback.playing && !resumeAfterScrub.current) {
      resumeAfterScrub.current = true;
      playback.pause();
    }
    playback.seek(next);
  };
  const commit = () => {
    if (!resumeAfterScrub.current) return;
    resumeAfterScrub.current = false;
    usePlayback.getState().play();
  };
  return (
    <div className="min-w-0 flex-1 px-1">
      <SingleSlider
        aria-label="Position"
        min={0}
        max={Math.max(1, duration - 1)}
        step={1}
        value={frame}
        onChange={seek}
        onValueCommitted={commit}
      />
    </div>
  );
}

function PreviewControls({
  player,
  fps,
  duration,
  fullscreen,
  container,
}: {
  player: PlayerRef | null;
  fps: number;
  duration: number;
  fullscreen: Fullscreen;
  container: HTMLElement | undefined;
}) {
  const playing = usePlayback((s) => s.playing);
  const frame = usePlayback((s) => s.frame);
  const timeAt = useFormatTime(fps);
  const sound = usePlayerVolume(player);
  return (
    <div className="@container flex h-9 shrink-0 items-center gap-1.5">
      <ControlButton
        label={playing ? "Pause" : "Play"}
        shortcut="Space"
        container={container}
        className="[&_svg]:fill-current"
        onClick={(e) => togglePlayback(player, e)}
      >
        {playing ? <Pause /> : <Play />}
      </ControlButton>
      <SeekBar duration={duration} />
      <span className="text-muted-foreground px-1 text-xs whitespace-nowrap tabular-nums">
        {timeAt(frame)} / {timeAt(duration)}
      </span>
      <ControlButton
        label={sound.muted ? "Unmute" : "Mute"}
        container={container}
        onClick={sound.toggleMute}
      >
        {sound.muted || sound.volume === 0 ? <VolumeX /> : <Volume2 />}
      </ControlButton>
      <div className="hidden w-20 shrink-0 pr-1 @md:block">
        <SingleSlider
          aria-label="Volume"
          min={0}
          max={1}
          step={0.01}
          value={sound.volume}
          onChange={sound.setVolume}
        />
      </div>
      <ControlButton
        label={fullscreen.active ? "Exit full screen" : "Full screen"}
        container={container}
        onClick={fullscreen.toggle}
      >
        {fullscreen.active ? <Minimize2 /> : <Maximize2 />}
      </ControlButton>
    </div>
  );
}

export function PreviewStage({
  player,
  fps,
  duration,
  children,
}: {
  player: PlayerRef | null;
  fps: number;
  duration: number;
  children: ReactNode;
}) {
  const [stage, setStage] = useState<HTMLDivElement | null>(null);
  const fullscreen = useFullscreen(stage);
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key !== " " || e.target !== e.currentTarget) return;
    e.preventDefault();
    togglePlayback(player, e);
  };
  return (
    <div
      ref={setStage}
      tabIndex={-1}
      onKeyDown={onKeyDown}
      className={cn(
        "flex min-h-0 flex-1 flex-col gap-1 outline-none",
        fullscreen.active && "bg-background p-3",
      )}
    >
      <div className="grid min-h-0 flex-1 place-items-center">{children}</div>
      <PreviewControls
        player={player}
        fps={fps}
        duration={duration}
        fullscreen={fullscreen}
        container={fullscreen.active && stage ? stage : undefined}
      />
    </div>
  );
}
