import { Sequence, useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useMemo } from "react";
import { launchScene } from "#video/elements/scene/launch-anything/content.ts";
import { LAUNCH_FPS, launchTimeline } from "#video/elements/scene/launch-anything/motion.ts";
import { Action } from "#video/elements/scene/launch-anything/scenes/action.tsx";
import { Address, ClosingMark } from "#video/elements/scene/launch-anything/scenes/closing.tsx";
import { Industry } from "#video/elements/scene/launch-anything/scenes/industry.tsx";
import { Opening } from "#video/elements/scene/launch-anything/scenes/opening.tsx";
import { Portal } from "#video/elements/scene/launch-anything/scenes/portal.tsx";
import { Proof } from "#video/elements/scene/launch-anything/scenes/proof.tsx";
import { Showcase } from "#video/elements/scene/launch-anything/scenes/showcase.tsx";
import { Space } from "#video/elements/scene/launch-anything/scenes/space.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

;

const scenes = {
  opening: Opening,
  action: Action,
  portal: Portal,
  showcase: Showcase,
  proof: Proof,
  industry: Industry,
  space: Space,
  address: Address,
  mark: ClosingMark,
};

export function LaunchAnything(settings: ElementSettings["launch-anything"]) {
  const frame = useFrame();
  const { width, height, fps } = useFilm();
  const scene = useMemo(() => launchScene(settings), [settings]);
  const scale = Math.min(width / 480, height / 270);
  return (
    <AbsoluteFill
      style={{
        background: scene.palette.night,
        color: scene.palette.lightText,
        overflow: "hidden",
        fontFamily: scene.look.font,
        WebkitFontSmoothing: "antialiased",
      }}
    >
      <div
        style={{
          position: "absolute",
          left: (width - 480 * scale) / 2,
          top: (height - 270 * scale) / 2,
          width: 480,
          height: 270,
          transform: `scale(${scale})`,
          transformOrigin: "0 0",
          overflow: "hidden",
        }}
      >
        {launchTimeline.map((shot) => {
          const Component = scenes[shot.id];
          return (
            <Sequence
              key={shot.id}
              name={shot.id}
              from={Math.round((shot.from / LAUNCH_FPS) * fps)}
              durationInFrames={
                Math.round((shot.to / LAUNCH_FPS) * fps) -
                Math.round((shot.from / LAUNCH_FPS) * fps)
              }
              layout="none"
            >
              <Component scene={scene} t={frame / fps} />
            </Sequence>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
