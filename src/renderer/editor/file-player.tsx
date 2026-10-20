import { useRef, type ComponentProps, type RefObject } from "react";
import {
  MediaPlayer,
  MediaPlayerAudio,
  MediaPlayerControls,
  MediaPlayerControlsOverlay,
  MediaPlayerError,
  MediaPlayerFullscreen,
  MediaPlayerLoading,
  MediaPlayerPlay,
  MediaPlayerPlaybackSpeed,
  MediaPlayerSeek,
  MediaPlayerTime,
  MediaPlayerVideo,
  MediaPlayerVolume,
} from "@/renderer/components/ui/media-player";
import { cn } from "@/renderer/lib/utils";

const ERROR_LABEL = "Couldn't play this file";
const ERROR_DESCRIPTION = "It may have been moved or deleted. Run the node again to make it.";

function PlaybackError({ media }: { media: RefObject<HTMLMediaElement | null> }) {
  return (
    <MediaPlayerError
      label={ERROR_LABEL}
      description={ERROR_DESCRIPTION}
      onReload={() => media.current?.load()}
    />
  );
}

export function VideoFilePlayer({
  src,
  label,
  muted,
  videoRef,
  className,
  videoClassName,
  onTimeUpdate,
}: {
  src: string;
  label: string;
  muted?: boolean;
  videoRef?: RefObject<HTMLVideoElement | null>;
  className?: string;
  videoClassName?: string;
  onTimeUpdate?: ComponentProps<"video">["onTimeUpdate"];
}) {
  const ownRef = useRef<HTMLVideoElement>(null);
  const ref = videoRef ?? ownRef;
  return (
    <MediaPlayer autoHide label={label} className={cn("h-auto bg-black", className)}>
      <MediaPlayerVideo
        ref={ref}
        src={src}
        muted={muted}
        playsInline
        className={cn("w-full", videoClassName)}
        onTimeUpdate={onTimeUpdate}
      />
      <MediaPlayerLoading />
      <PlaybackError media={ref} />
      <MediaPlayerControls className="flex-col items-start gap-1.5">
        <MediaPlayerControlsOverlay />
        <MediaPlayerSeek withoutChapter />
        <div className="flex w-full items-center gap-1">
          <MediaPlayerPlay />
          <MediaPlayerVolume expandable />
          <MediaPlayerTime className="text-xs" />
          <div className="flex-1" />
          <MediaPlayerPlaybackSpeed />
          <MediaPlayerFullscreen />
        </div>
      </MediaPlayerControls>
    </MediaPlayer>
  );
}

export function AudioFilePlayer({ src, label }: { src: string; label: string }) {
  const ref = useRef<HTMLAudioElement>(null);
  return (
    <MediaPlayer label={label} className="h-auto bg-muted/40">
      <MediaPlayerAudio ref={ref} src={src} className="sr-only" />
      <PlaybackError media={ref} />
      <MediaPlayerControls className="relative flex-col items-start gap-1.5">
        <MediaPlayerSeek withoutChapter />
        <div className="flex w-full items-center gap-1">
          <MediaPlayerPlay />
          <MediaPlayerVolume expandable />
          <MediaPlayerTime className="text-xs" />
          <div className="flex-1" />
          <MediaPlayerPlaybackSpeed />
        </div>
      </MediaPlayerControls>
    </MediaPlayer>
  );
}
