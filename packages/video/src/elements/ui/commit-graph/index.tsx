import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

const T = {
  window: 0,
  windowFor: 0.46,
  chrome: 0.14,
  exitFor: 0.42,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

export function CommitGraph({
  commits: commitsProp,
  windowTitle,
  startAtSeconds,
  stepSeconds,
  accentColor,
  laneColors,
  backgroundColor,
  theme,
  speed,
  codeFont,
  font,
  windowWidth,
  rowSpacing,
  laneSpacing,
  dotSize,
  lineWidth,
  textSize,
  cornerRadius,
  windowDots,
  holdSeconds,
}: ElementSettings["commit-graph"] & { holdSeconds?: number }) {
  const fontFamily = fontOr(font, "main");
  const mono = fontOr(codeFont, "code");
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = codePalette(theme);
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const portrait = height > width;
  const designU = portrait
    ? Math.min(width / 496, height / 896)
    : Math.min(width / 1024, height / 576);

  const commits = commitsProp.map((commit, index) => {
    const earlier = (ref: number | undefined) =>
      ref !== undefined && Number.isInteger(ref) && ref >= 0 && ref < index ? ref : undefined;
    return {
      ...commit,
      lane: Math.max(0, Math.round(commit.lane ?? 0)),
      parent: earlier(commit.parent),
      mergeFrom: earlier(commit.mergeFrom),
    };
  });
  const u = Math.min(
    designU,
    (height - safe.paddingTop - safe.paddingBottom) /
      ((windowTitle ? 40 : 0) + 20 + 48 * commits.length),
  );

  const windowIn = spring({
    frame: frame - at(T.window),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.85 },
  });
  const open = ease(T.window, T.window + T.windowFor, EASING.editorial);
  const chromeIn = ease(T.chrome, T.chrome + 0.38);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  const windowW = Math.min(width - safe.paddingLeft - safe.paddingRight, windowWidth * u);
  const rowH = rowSpacing * u;
  const laneW = laneSpacing * u;
  const gutterX = 26 * u;
  const lanes = commits.reduce((max, commit) => Math.max(max, commit.lane ?? 0), 0) + 1;
  const graphW = gutterX + lanes * laneW;
  const bodyH = rowH * commits.length;
  const dotR = dotSize * u;

  const laneColor = (lane: number) =>
    lane === 0 ? accentColor : laneColors[(lane - 1) % laneColors.length];

  const rowY = (index: number) => index * rowH + rowH / 2;
  const laneX = (lane: number) => gutterX + lane * laneW;

  const arrivalOf = (index: number) =>
    ease(
      startAtSeconds + index * stepSeconds,
      startAtSeconds + (index + 1) * stepSeconds,
      EASING.editorial,
    );

  const edgePath = (fromIndex: number, toIndex: number) => {
    const from = commits[fromIndex];
    const to = commits[toIndex];
    const x1 = laneX(from.lane ?? 0);
    const y1 = rowY(fromIndex);
    const x2 = laneX(to.lane ?? 0);
    const y2 = rowY(toIndex);
    if (x1 === x2) return `M ${x1} ${y1} L ${x2} ${y2}`;
    const bend = (y2 - y1) * 0.55;
    return `M ${x1} ${y1} C ${x1} ${y1 + bend}, ${x2} ${y2 - bend}, ${x2} ${y2}`;
  };

  return (
    <AbsoluteFill
      style={{
        background: backgroundColor || palette.page,
        fontFamily,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: `${safe.paddingTop}px ${safe.paddingRight}px`,
      }}
    >
      <div
        style={{
          width: windowW,
          borderRadius: cornerRadius * u,
          background: palette.window,
          border: `1px solid ${palette.border}`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${24 * u}px ${
            60 * u
          }px ${palette.shadow}`,
          overflow: "hidden",
          clipPath: `inset(0 0 ${(1 - open) * 100}% 0 round ${18 * u}px)`,
          opacity: 1 - exit,
          translate: `0 ${(1 - windowIn) * 22 * u + exit * 30 * u}px`,
        }}
      >
        {windowTitle ? (
          <div
            style={{
              height: 40 * u,
              display: "flex",
              alignItems: "center",
              gap: 8 * u,
              padding: `0 ${16 * u}px`,
              background: palette.header,
              borderBottom: `1px solid ${palette.border}`,
              opacity: chromeIn,
            }}
          >
            {(windowDots ? [0, 1, 2] : []).map((dot) => (
              <span
                key={dot}
                style={{
                  width: 9 * u,
                  height: 9 * u,
                  borderRadius: 999,
                  background: palette.faint,
                }}
              />
            ))}
            <span
              style={{
                marginLeft: 10 * u,
                color: palette.dim,
                fontSize: textSize * (14.5 / 15.5) * u,
                fontWeight: 500,
                fontFamily: mono,
              }}
            >
              {windowTitle}
            </span>
          </div>
        ) : null}

        <div style={{ position: "relative", height: bodyH, padding: `${10 * u}px 0` }}>
          <svg
            width={graphW}
            height={bodyH}
            viewBox={`0 0 ${graphW} ${bodyH}`}
            style={{ position: "absolute", left: 18 * u, top: 10 * u }}
          >
            {commits.map((commit, index) => {
              if (index === 0) return null;
              const progress = arrivalOf(index);
              if (progress <= 0) return null;
              const parent = commit.parent ?? index - 1;
              const lane = commit.lane ?? 0;
              const edges = [{ from: parent, stroke: laneColor(lane) }];
              if (commit.mergeFrom !== undefined) {
                edges.push({
                  from: commit.mergeFrom,
                  stroke: laneColor(commits[commit.mergeFrom].lane ?? 0),
                });
              }
              return edges.map((edge, edgeIndex) => (
                <path
                  key={edgeIndex}
                  d={edgePath(edge.from, index)}
                  fill="none"
                  stroke={edge.stroke}
                  strokeWidth={lineWidth * u}
                  strokeLinecap="round"
                  opacity={0.85}
                  pathLength={1}
                  strokeDasharray={1}
                  strokeDashoffset={1 - progress}
                />
              ));
            })}

            {commits.map((commit, index) => {
              const progress =
                index === 0 ? ease(startAtSeconds - 0.24, startAtSeconds) : arrivalOf(index);
              const pop = interpolate(progress, [0.82, 1], [0, 1], clamp);
              if (pop <= 0) return null;
              const color = laneColor(commit.lane ?? 0);
              return (
                <circle
                  key={index}
                  cx={laneX(commit.lane ?? 0)}
                  cy={rowY(index)}
                  r={dotR * (0.6 + pop * 0.4)}
                  fill={palette.window}
                  stroke={color}
                  strokeWidth={lineWidth * (2.6 / 2.2) * u}
                  opacity={pop}
                />
              );
            })}
          </svg>

          {commits.map((commit, index) => {
            const progress =
              index === 0 ? ease(startAtSeconds - 0.24, startAtSeconds) : arrivalOf(index);
            const textIn = interpolate(progress, [0.86, 1], [0, 1], clamp);
            return (
              <div
                key={index}
                style={{
                  position: "absolute",
                  left: 18 * u + graphW + 14 * u,
                  right: 20 * u,
                  top: 10 * u + rowY(index) - 12 * u,
                  height: 24 * u,
                  display: "flex",
                  alignItems: "center",
                  gap: 12 * u,
                  opacity: textIn,
                  translate: `${(1 - textIn) * 14 * u}px`,
                }}
              >
                <span
                  style={{
                    color: laneColor(commit.lane ?? 0),
                    fontSize: textSize * (14.5 / 15.5) * u,
                    fontWeight: 600,
                    fontFamily: mono,
                    flexShrink: 0,
                  }}
                >
                  {commit.hash}
                </span>
                <span
                  style={{
                    color: palette.fg,
                    fontSize: textSize * u,
                    fontWeight: 500,
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {commit.message}
                </span>
                {commit.ref ? (
                  <span
                    style={{
                      flexShrink: 0,
                      padding: `${2.5 * u}px ${8 * u}px`,
                      borderRadius: 6 * u,
                      background: `color-mix(in srgb, ${accentColor} 13%, transparent)`,
                      border: `1px solid color-mix(in srgb, ${accentColor} 40%, transparent)`,
                      color: accentColor,
                      fontSize: textSize * (14.5 / 15.5) * u,
                      fontWeight: 600,
                    }}
                  >
                    {commit.ref}
                  </span>
                ) : null}
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}
