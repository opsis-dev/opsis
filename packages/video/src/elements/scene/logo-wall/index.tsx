import { AbsoluteFill, Img, interpolate, useCurrentFrame, useVideoConfig } from "remotion";
import { CODE_THEMES } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const fontFamily = '"Inter"';

export type WallLogo = {
  name: string;
  color?: string;
  glyph?: string;
  image?: string;
};

export type LogoWallProps = {
  logos?: WallLogo[];
  eyebrow?: string;
  title?: string;
  columns?: number;
  startAtSeconds?: number;
  arriveStaggerSeconds?: number;
  staggerSeconds?: number;
  warmSeconds?: number;
  holdSeconds?: number;
  accentColor?: string;
  backgroundColor?: string;
  theme?: "dark" | "light";
  speed?: number;
};

const DEFAULT_LOGOS: WallLogo[] = [
  { name: "Northstar", color: "#E8B86D" },
  { name: "Halcyon", color: "#7DD3E8" },
  { name: "Fernweh", color: "#9BD4A0" },
  { name: "Orbital", color: "#C99BE8" },
  { name: "Lumen", color: "#8FB8F0" },
  { name: "Tidewater", color: "#E89B9B" },
  { name: "Kestrel", color: "#E8B86D" },
  { name: "Marrow", color: "#7DD3E8" },
];

const T = {
  head: 0.08,
  headFor: 0.4,
  exitFor: 0.42,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

export const LogoWall: React.FC<LogoWallProps> = ({
  logos = DEFAULT_LOGOS,
  eyebrow = "Trusted by",
  title = "Teams that ship with us",
  columns = 4,
  startAtSeconds = 0.32,
  arriveStaggerSeconds = 0.06,
  staggerSeconds = 0.24,
  warmSeconds = 0.6,
  holdSeconds,
  accentColor = "#E8B86D",
  backgroundColor,
  theme = "dark",
  speed = 1,
}) => {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = CODE_THEMES[theme];
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const portrait = height > width;
  const u = portrait ? Math.min(width / 496, height / 896) : Math.min(width / 1024, height / 576);

  const headIn = ease(T.head, T.head + T.headFor);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  const wallW = Math.min(width - safe.paddingLeft - safe.paddingRight, 900 * u);
  const gap = 14 * u;
  const perRow = Math.max(1, Math.round(columns));
  const tileW = (wallW - gap * (perRow - 1)) / perRow;
  const tileH = 88 * u;

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor ?? palette.page,
        fontFamily,
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
              textTransform: "uppercase",
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
              fontSize: 30 * u,
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
            const markWarm = interpolate(warm, [0, 0.7], [0, 1], clamp);
            const color = logo.color ?? accentColor;

            return (
              <div
                key={index}
                style={{
                  width: tileW,
                  height: tileH,
                  borderRadius: 14 * u,
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
                  filter: `saturate(${warm}) brightness(${0.72 + warm * 0.28})`,
                }}
              >
                {logo.image ? (
                  <Img
                    src={logo.image}
                    style={{
                      maxWidth: "78%",
                      maxHeight: 46 * u,
                      objectFit: "contain",
                    }}
                  />
                ) : (
                  <>
                    <div
                      style={{
                        width: 30 * u,
                        height: 30 * u,
                        flexShrink: 0,
                        borderRadius: 9 * u,
                        display: "grid",
                        placeItems: "center",
                        background: `linear-gradient(140deg, ${color}, color-mix(in srgb, ${color} 53%, transparent))`,
                        color: theme === "dark" ? "#12100C" : "#FFFFFF",
                        fontSize: 16 * u,
                        fontWeight: 800,
                        opacity: 0.45 + markWarm * 0.55,
                      }}
                    >
                      {logo.glyph ?? logo.name[0]}
                    </div>
                    <span
                      style={{
                        color,
                        fontSize: 19 * u,
                        fontWeight: 700,
                        letterSpacing: "-0.02em",
                        minWidth: 0,
                        maxWidth: tileW - 30 * u - 10 * u - 24 * u,
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
};
