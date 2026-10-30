import { Interactive, interpolate } from "remotion";
import { CLAMPED } from "#video/core/anim.ts";
import type { ReleaseTeaserScene } from "#video/elements/scene/release-teaser/content.ts";
import { ease, fitText } from "#video/elements/scene/release-teaser/motion.ts";
import { BrandImage, OrbitMark } from "#video/elements/scene/release-teaser/ui.tsx";

export function ReleaseLockup({ scene, frame }: { scene: ReleaseTeaserScene; frame: number }) {
  const title = `${scene.brandName}${scene.release ? ` ${scene.release}` : ""}`;
  const reveal = scene.reducedMotion ? 1 : ease(frame, 0, 32);
  return (
    <Interactive.Div
      name="Release lockup"
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        opacity: reveal,
        filter: !scene.reducedMotion && reveal < 0.99 ? `blur(${(1 - reveal) * 6}px)` : undefined,
        scale: scene.reducedMotion ? 1 : interpolate(frame, [0, 350], [0.97, 1.03], CLAMPED),
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 16,
          marginBottom: 16,
        }}
      >
        <div style={{ width: scene.logoSize, height: scene.logoSize, flexShrink: 0 }}>
          {scene.logoSrc ? (
            <BrandImage
              src={scene.logoSrc}
              style={{ width: scene.logoSize, height: scene.logoSize, objectFit: "contain" }}
            />
          ) : (
            <OrbitMark size={scene.logoSize} color={scene.theme.accent} />
          )}
        </div>
        <Interactive.Div
          name="Brand and release"
          style={{
            fontSize: fitText(title, 70, 750) * scene.textSize,
            fontWeight: Math.min(900, scene.statementWeight + 100),
            lineHeight: 1,
            color: scene.theme.foreground,
            whiteSpace: "pre",
            textAlign: "center",
          }}
        >
          {title}
        </Interactive.Div>
      </div>
      <Interactive.Div
        name="Closing line"
        style={{
          fontSize: fitText(scene.tagline, 20, 740) * scene.textSize,
          lineHeight: 1.3,
          color: scene.theme.muted,
          whiteSpace: "pre",
          textAlign: "center",
          opacity: scene.reducedMotion ? 1 : ease(frame, 8, 40),
        }}
      >
        {scene.tagline}
      </Interactive.Div>
    </Interactive.Div>
  );
}
