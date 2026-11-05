import { Sequence, useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useMemo } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { TeaserBackground } from "#video/elements/scene/release-teaser/background.tsx";
import { releaseTeaserScene } from "#video/elements/scene/release-teaser/content.ts";
import { atFps, RELEASE_TEASER_FPS } from "#video/elements/scene/release-teaser/motion.ts";
import { ReleaseLockup } from "#video/elements/scene/release-teaser/scenes/closing.tsx";
import { Statement } from "#video/elements/scene/release-teaser/scenes/statement.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

;

export function ReleaseTeaser(settings: ElementSettings["release-teaser"]) {
  const currentFrame = useFrame();
  const { fps, width, height } = useFilm();
  const frame = (currentFrame / fps) * RELEASE_TEASER_FPS;
  const scene = useMemo(() => releaseTeaserScene(settings), [settings]);
  const scale = Math.min(width / 960, height / 540);
  return (
    <AbsoluteFill
      style={{
        background: scene.theme.background,
        overflow: "hidden",
        fontFamily: fontOr(settings.fontFamily, "main"),
        WebkitFontSmoothing: "antialiased",
      }}
    >
      <div
        style={{
          position: "absolute",
          left: (width - 960 * scale) / 2,
          top: (height - 540 * scale) / 2,
          width: 960,
          height: 540,
          scale,
          transformOrigin: "0 0",
          overflow: "hidden",
        }}
      >
        <TeaserBackground scene={scene} frame={frame} />
        <Sequence name="An idea" from={0} durationInFrames={atFps(104, fps)} layout="none">
          <Statement scene={scene} text={scene.statements[0]} frame={frame} duration={104} />
        </Sequence>
        <Sequence
          name="Better work"
          from={atFps(104, fps)}
          durationInFrames={atFps(208, fps) - atFps(104, fps)}
          layout="none"
        >
          <Statement scene={scene} text={scene.statements[1]} frame={frame - 104} duration={104} />
        </Sequence>
        <Sequence
          name="Every detail"
          from={atFps(208, fps)}
          durationInFrames={atFps(326, fps) - atFps(208, fps)}
          layout="none"
        >
          <Statement scene={scene} text={scene.statements[2]} frame={frame - 208} duration={118} />
        </Sequence>
        <Sequence
          name="Together"
          from={atFps(326, fps)}
          durationInFrames={atFps(446, fps) - atFps(326, fps)}
          layout="none"
        >
          <Statement scene={scene} text={scene.statements[3]} frame={frame - 326} duration={120} />
        </Sequence>
        <Sequence
          name="A fresh chapter"
          from={atFps(446, fps)}
          durationInFrames={atFps(610, fps) - atFps(446, fps)}
          layout="none"
        >
          <Statement scene={scene} text={scene.statements[4]} frame={frame - 446} duration={164} />
        </Sequence>
        <Sequence
          name="Release reveal"
          from={atFps(610, fps)}
          durationInFrames={atFps(960, fps) - atFps(610, fps)}
          layout="none"
        >
          <ReleaseLockup scene={scene} frame={frame - 610} />
        </Sequence>
      </div>
    </AbsoluteFill>
  );
}
