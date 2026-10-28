import { useMemo } from "react";
import type React from "react";
import { AbsoluteFill, Audio, Sequence, useCurrentFrame, useVideoConfig } from "remotion";
import { type ElementProps } from "#video/common/spec.ts";
import { TeaserBackground } from "#video/elements/scene/release-teaser/background.tsx";
import {
  type ReleaseTeaserProps,
  resolveReleaseTeaserProps,
} from "#video/elements/scene/release-teaser/content.ts";
import { atFps, RELEASE_TEASER_FPS } from "#video/elements/scene/release-teaser/motion.ts";
import { ReleaseLockup } from "#video/elements/scene/release-teaser/scenes/closing.tsx";
import { Statement } from "#video/elements/scene/release-teaser/scenes/statement.tsx";
import { nestedFrom } from "#video/lib/settings/shape.ts";

export type {
  ReleaseTeaserProps,
  ReleaseTeaserTheme,
} from "#video/elements/scene/release-teaser/content.ts";
export {
  releaseTeaserStatements,
  releaseTeaserTheme,
} from "#video/elements/scene/release-teaser/content.ts";
export {
  RELEASE_TEASER_FPS,
  releaseTeaserTimeline,
} from "#video/elements/scene/release-teaser/motion.ts";

function ReleaseTeaserView(props: ReleaseTeaserProps) {
  const currentFrame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const frame = (currentFrame / fps) * RELEASE_TEASER_FPS;
  const {
    brandName,
    release,
    tagline,
    statements,
    accentColor,
    theme,
    logoSrc,
    lightIntensity,
    reducedMotion,
  } = props;
  const scene = useMemo(
    () =>
      resolveReleaseTeaserProps({
        brandName,
        release,
        tagline,
        statements,
        accentColor,
        theme,
        logoSrc,
        lightIntensity,
        reducedMotion,
      }),
    [
      brandName,
      release,
      tagline,
      statements,
      accentColor,
      theme,
      logoSrc,
      lightIntensity,
      reducedMotion,
    ],
  );
  const scale = Math.min(width / 960, height / 540);
  return (
    <AbsoluteFill
      style={{
        background: scene.theme.background,
        overflow: "hidden",
        fontFamily: '"Inter", sans-serif',
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
      {props.audioSrc?.trim() ? (
        <Audio
          src={props.audioSrc.trim()}
          volume={Number.isFinite(props.volume) ? Math.max(0, Math.min(1, props.volume ?? 1)) : 1}
        />
      ) : null}
    </AbsoluteFill>
  );
}
export function ReleaseTeaser(props: ElementProps & { children?: React.ReactNode }) {
  return (
    <ReleaseTeaserView
      {...(nestedFrom(props, {
        themeBackground: "theme.background",
        themeSurface: "theme.surface",
        themeForeground: "theme.foreground",
        themeMuted: "theme.muted",
      }) as unknown as React.ComponentProps<typeof ReleaseTeaserView>)}
    />
  );
}
