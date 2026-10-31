import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill, Img } from "@opsis/runtime/parts.tsx";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const T = {
  head: 0.08,
  headFor: 0.4,
  exitFor: 0.42,
} as const;

export function LogoWall({
  logos,
  eyebrow,
  title,
  columns,
  startAtSeconds,
  arriveStaggerSeconds,
  staggerSeconds,
  warmSeconds,
  holdSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  glyphColor,
  fontFamily,
  wallWidth,
  gap: gapPx,
  tileHeight,
  tileRadius,
  titleSize,
  eyebrowUppercase,
  nameSize,
  imageSize,
  showMark,
  startBrightness,
}: ElementSettings["logo-wall"] & { holdSeconds?: number }) {
  const frame = useFrame();
  const { fps, width, height } = useFilm();
  const palette = codePalette(theme);
  const safe = getSafeAreaPadding({ width, height });

  function at(seconds: number) {
    return (seconds * fps) / speed;
  }
  function ease(from: number, to: number, easing = EASING.enter) {
    return interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...CLAMPED });
  }

  const portrait = height > width;
  const u = portrait ? Math.min(width / 496, height / 896) : Math.min(width / 1024, height / 576);

  const headIn = ease(T.head, T.head + T.headFor);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  const wallW = Math.min(width - safe.paddingLeft - safe.paddingRight, wallWidth * u);
  const gap = gapPx * u;
  const perRow = Math.max(1, Math.round(columns));
  const tileW = (wallW - gap * (perRow - 1)) / perRow;
  const tileH = tileHeight * u;

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor || palette.page,
        fontFamily: fontOr(fontFamily, "main"),
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: `${safe.paddingTop}px ${safe.paddingRight}px`,
      }}
    >
      <div
        style={{
          width: wallW,
          opacity: 1 - exit,
          translate: `0 ${exit * 26 * u}px`,
        }}
      >
        {eyebrow ? (
          <div
            style={{
              color: accentColor,
              fontSize: 14.5 * u,
              fontWeight: 600,
              letterSpacing: "0.14em",
              textTransform: eyebrowUppercase ? "uppercase" : "none",
              opacity: headIn,
            }}
          >
            {eyebrow}
          </div>
        ) : null}
        {title ? (
          <div
            style={{
              marginTop: 8 * u,
              marginBottom: 24 * u,
              color: palette.fg,
              fontSize: titleSize * u,
              fontWeight: 700,
              letterSpacing: "-0.02em",
              opacity: headIn,
              translate: `0 ${(1 - headIn) * 12 * u}px`,
            }}
          >
            {title}
          </div>
        ) : null}

        <div style={{ display: "flex", flexWrap: "wrap", gap }}>
          {logos.map((logo, index) => {
            const row = Math.floor(index / perRow);
            const column = index % perRow;
            const arriveAt = startAtSeconds + index * arriveStaggerSeconds;
            const warmAt = startAtSeconds + 0.3 + (row + column) * staggerSeconds;
            const arrive = ease(arriveAt, arriveAt + 0.36);
            const warm = ease(warmAt, warmAt + warmSeconds, EASING.editorial);
            const markWarm = interpolate(warm, [0, 0.7], [0, 1], CLAMPED);
            const color = logo.color ?? accentColor;

            return (
              <div
                key={index}
                style={{
                  width: tileW,
                  height: tileH,
                  borderRadius: tileRadius * u,
                  background: palette.band,
                  border: `1px solid ${
                    warm > 0.5 ? `color-mix(in srgb, ${color} 27%, transparent)` : palette.border
                  }`,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  gap: 10 * u,
                  opacity: arrive,
                  scale: `${0.94 + arrive * 0.06}`,
                  filter: `saturate(${warm}) brightness(${startBrightness + warm * (1 - startBrightness)})`,
                }}
              >
                {logo.image ? (
                  <Img
                    src={logo.image}
                    style={{
                      maxWidth: "78%",
                      maxHeight: imageSize * u,
                      objectFit: "contain",
                    }}
                  />
                ) : (
                  <>
                    {showMark ? (
                      <div
                        style={{
                          width: 30 * u,
                          height: 30 * u,
                          flexShrink: 0,
                          borderRadius: 9 * u,
                          display: "grid",
                          placeItems: "center",
                          background: `linear-gradient(140deg, ${color}, color-mix(in srgb, ${color} 53%, transparent))`,
                          color: glyphColor || (theme.mode === "dark" ? "#12100C" : "#FFFFFF"),
                          fontSize: 16 * u,
                          fontWeight: 800,
                          opacity: 0.45 + markWarm * 0.55,
                        }}
                      >
                        {logo.glyph ?? logo.name[0]}
                      </div>
                    ) : null}
                    <span
                      style={{
                        color,
                        fontSize: nameSize * u,
                        fontWeight: 700,
                        letterSpacing: "-0.02em",
                        minWidth: 0,
                        maxWidth: tileW - (showMark ? 40 * u : 0) - 24 * u,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {logo.name}
                    </span>
                  </>
                )}
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}
