import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "@remotion/transitions";
import type React from "react";
import { AbsoluteFill, useVideoConfig } from "remotion";

const LIFT_RATIO = 1.28;

function originFractions(origin: string): [number, number] {
  const parts = origin.trim().split(/\s+/).map((part) => {
    const value = Number.parseFloat(part);
    return part.endsWith("%") && Number.isFinite(value) ? value / 100 : null;
  });
  return [parts[0] ?? 0.5, parts[1] ?? 0.5];
}

export function pageTurnLift(width: number, height: number, origin: string, angle: number): number {
  const [ox, oy] = originFractions(origin);
  const theta = (angle * Math.PI) / 180;
  const lowest = [
    [0, 0],
    [width, 0],
    [0, height],
    [width, height],
  ].reduce((low, [x, y]) => {
    const dx = x - ox * width;
    const dy = y - oy * height;
    return Math.max(low, oy * height + dx * Math.sin(theta) + dy * Math.cos(theta));
  }, 0);
  return Math.max(height * LIFT_RATIO, lowest * 1.02);
}

export function pageTurnPose(progress: number, poses: number): number {
  const clamped = Math.min(1, Math.max(0, progress));
  const eased = clamped ** 3;
  if (poses <= 1) return eased >= 1 ? 1 : 0;
  const bucket = Math.min(poses - 1, Math.floor(eased * poses));
  return bucket / (poses - 1);
}

export type PageTurnProps = {
  angle?: number;
  origin?: string;
  poses?: number;
};

const PageTurnPresentation: React.FC<
  TransitionPresentationComponentProps<PageTurnProps>
> = ({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}) => {
  const { width, height } = useVideoConfig();
  const { angle = -7, origin = "18% 100%", poses = 8 } = passedProps;

  if (presentationDirection === "entering") {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  const pose = pageTurnPose(presentationProgress, poses);

  return (
    <AbsoluteFill
      style={{
        zIndex: 2,
        transformOrigin: origin,
        translate: `0 ${-pose * pageTurnLift(width, height, origin, angle)}px`,
        rotate: `${angle * pose}deg`,
      }}
    >
      {children}
    </AbsoluteFill>
  );
};

export function pageTurn(
  props: PageTurnProps = {},
): TransitionPresentation<PageTurnProps> {
  return {
    component: PageTurnPresentation,
    props,
  };
}
