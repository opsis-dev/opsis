import type { CSSProperties, ReactNode } from "react";
import {
  Freeze as RemotionFreeze,
  Loop as RemotionLoop,
  Sequence as RemotionSequence,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

export type Film = {
  width: number;
  height: number;
  fps: number;
  durationInFrames: number;
};

export function useFrame(): number {
  return useCurrentFrame();
}

export function useFilm(): Film {
  const { width, height, fps, durationInFrames } = useVideoConfig();
  return { width, height, fps, durationInFrames };
}

export type SequenceProps = {
  from?: number;
  durationInFrames?: number;
  layout?: "absolute-fill" | "none";
  width?: number;
  height?: number;
  name?: string;
  premountFor?: number;
  style?: CSSProperties;
  children?: ReactNode;
};

export function Sequence(props: SequenceProps) {
  return <RemotionSequence {...props} />;
}

export function Freeze({ frame, children }: { frame: number; children?: ReactNode }) {
  return <RemotionFreeze frame={frame}>{children}</RemotionFreeze>;
}

export function Loop({
  durationInFrames,
  children,
}: {
  durationInFrames: number;
  children?: ReactNode;
}) {
  return <RemotionLoop durationInFrames={durationInFrames}>{children}</RemotionLoop>;
}
