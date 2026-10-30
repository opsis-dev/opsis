import { AbsoluteFill, useVideoConfig } from "remotion";
import type { ElementSettings } from "#video/generated/spec.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";

const LIFT_RATIO = 1.28;

const HINGE_Y = 1;

export function pageTurnLift(width: number, height: number, ox: number, angle: number): number {
  const oy = HINGE_Y;
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

type PageTurnProps = ElementSettings["page-turn"];

function PageTurnPresentation({
  children,
  presentationProgress,
  presentationDirection,
  passedProps,
}: TransitionPresentationComponentProps<PageTurnProps>) {
  const { width, height } = useVideoConfig();
  const { angle, poses, hingeX } = passedProps;

  if (presentationDirection === "entering") {
    return <AbsoluteFill>{children}</AbsoluteFill>;
  }

  const pose = pageTurnPose(presentationProgress, poses);

  return (
    <AbsoluteFill
      style={{
        zIndex: 2,
        transformOrigin: `${hingeX * 100}% ${HINGE_Y * 100}%`,
        translate: `0 ${-pose * pageTurnLift(width, height, hingeX, angle)}px`,
        rotate: `${angle * pose}deg`,
      }}
    >
      {children}
    </AbsoluteFill>
  );
}

export function pageTurn(props: PageTurnProps): TransitionPresentation<PageTurnProps> {
  return {
    component: PageTurnPresentation,
    props,
  };
}
