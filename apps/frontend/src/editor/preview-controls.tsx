import { Maximize2, Minimize2, Pause, Play, Volume2, VolumeX } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { Kbd } from "@/components/ui/kbd";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { SingleSlider } from "../app/single-slider";
import { usePreviewPlayback } from "./preview-playback";
import type { StemSound } from "./stem-player";
import { useFormatTime } from "./timeline/timeline-state";

type Fullscreen = { active: boolean; toggle: () => void };

export function togglePlayback(sound: StemSound) {
  const playback = usePreviewPlayback.getState();
  if (playback.playing) {
    playback.pause();
    return;
  }
  playback.play();
  sound.start();
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
  const frame = usePreviewPlayback((s) => s.frame);
  const resumeAfterScrub = useRef(false);
  const seek = (next: number) => {
    const playback = usePreviewPlayback.getState();
    if (playback.playing && !resumeAfterScrub.current) {
      resumeAfterScrub.current = true;
      playback.pause();
    }
    playback.seek(next);
  };
  const commit = () => {
    if (!resumeAfterScrub.current) return;
    resumeAfterScrub.current = false;
    usePreviewPlayback.getState().play();
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
  sound,
  fps,
  duration,
  fullscreen,
  container,
}: {
  sound: StemSound;
  fps: number;
  duration: number;
  fullscreen: Fullscreen;
  container: HTMLElement | undefined;
}) {
  const playing = usePreviewPlayback((s) => s.playing);
  const frame = usePreviewPlayback((s) => s.frame);
  const timeAt = useFormatTime(fps);
  return (
    <div className="@container flex h-9 shrink-0 items-center gap-1.5">
      <ControlButton
        label={playing ? "Pause" : "Play"}
        shortcut="Space"
        container={container}
        className="[&_svg]:fill-current"
        onClick={() => togglePlayback(sound)}
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
  sound,
  fps,
  duration,
  onStage,
  children,
}: {
  sound: StemSound;
  fps: number;
  duration: number;
  onStage: (stage: HTMLDivElement | null) => void;
  children: ReactNode;
}) {
  const [stage, setStage] = useState<HTMLDivElement | null>(null);
  const [view, setView] = useState<HTMLDivElement | null>(null);
  useEffect(() => onStage(view), [view, onStage]);
  const fullscreen = useFullscreen(stage);
  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key !== " " || e.target !== e.currentTarget) return;
    e.preventDefault();
    togglePlayback(sound);
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
      <div ref={setView} className="grid min-h-0 flex-1 place-items-center">
        {children}
      </div>
      <PreviewControls
        sound={sound}
        fps={fps}
        duration={duration}
        fullscreen={fullscreen}
        container={fullscreen.active && stage ? stage : undefined}
      />
    </div>
  );
}
