import { AbsoluteFill } from "remotion";
import type { ElementSettings, WipeDirection } from "#video/generated/spec.ts";
import type { TransitionPresentationComponentProps } from "#video/lib/transitions/presentation.ts";
import { presentation } from "#video/registry/framing.tsx";

type WipeProps = { direction: WipeDirection };

type Corner = [number, number];

function edgeWipe(direction: WipeDirection, shown: number): Corner[] | null {
  switch (direction) {
    case "from-left":
      return [
        [0, 0],
        [shown, 0],
        [shown, 1],
        [0, 1],
      ];
    case "from-right":
      return [
        [1 - shown, 0],
        [1, 0],
        [1, 1],
        [1 - shown, 1],
      ];
    case "from-top":
      return [
        [0, 0],
        [1, 0],
        [1, shown],
        [0, shown],
      ];
    case "from-bottom":
      return [
        [0, 1 - shown],
        [1, 1 - shown],
        [1, 1],
        [0, 1],
      ];
    default:
      return null;
  }
}

function cornerWipe(direction: WipeDirection, shown: number): Corner[] {
  const reach = shown * 2;
  const [x, y] = {
    "from-top-left": [0, 0],
    "from-top-right": [1, 0],
    "from-bottom-left": [0, 1],
    "from-bottom-right": [1, 1],
  }[direction as Exclude<WipeDirection, "from-left" | "from-right" | "from-top" | "from-bottom">];
  const across = x === 0 ? reach : 1 - reach;
  const down = y === 0 ? reach : 1 - reach;
  return [
    [x, y],
    [across, y],
    [x, down],
  ];
}

export function wipeCorners(direction: WipeDirection, progress: number): Corner[] {
  return edgeWipe(direction, progress) ?? cornerWipe(direction, progress);
}

function Wipe({
  children,
  presentationProgress,
  presentationDirection,
  passedProps: { direction },
}: TransitionPresentationComponentProps<WipeProps>) {
  if (presentationDirection === "exiting") return <AbsoluteFill>{children}</AbsoluteFill>;
  const corners = wipeCorners(direction, presentationProgress)
    .map(([x, y]) => `${x * 100}% ${y * 100}%`)
    .join(", ");
  return <AbsoluteFill style={{ clipPath: `polygon(${corners})` }}>{children}</AbsoluteFill>;
}

export const wipe = presentation(({ direction }: ElementSettings["wipe"]) => ({
  component: Wipe,
  props: { direction },
}));
