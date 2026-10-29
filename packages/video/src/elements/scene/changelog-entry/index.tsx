import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type ChangeKind = "added" | "fixed" | "changed" | "removed";

export type ChangeRow = {
  kind: ChangeKind;
  text: string;
};

const DEFAULT_KIND_COLORS: Record<ChangeKind, string> = {
  added: "#9BD4A0",
  fixed: "#7DD3E8",
  changed: "#E8B86D",
  removed: "#E89B9B",
};

const T = {
  card: 0,
  cardFor: 0.46,
  version: 0.12,
  versionFor: 0.44,
  rule: 0.34,
  ruleFor: 0.5,
  exitFor: 0.42,
} as const;

const clamp = {
  extrapolateLeft: "clamp",
  extrapolateRight: "clamp",
} as const;

function ChangelogEntryView({
  version,
  date,
  summary,
  changes,
  kindColors,
  startAtSeconds,
  staggerSeconds,
  holdSeconds,
  backgroundColor,
  theme,
  speed,
  tags,
  cardWidth,
  fontFamily,
}: Omit<
  ElementSettings["changelog-entry"],
  | "kindAdded"
  | "kindFixed"
  | "kindChanged"
  | "kindRemoved"
  | "addedLabel"
  | "fixedLabel"
  | "changedLabel"
  | "removedLabel"
> & {
  kindColors: Partial<Record<ChangeKind, string>>;
  tags: Record<ChangeKind, string>;
  holdSeconds?: number;
}) {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
  const palette = codePalette(theme);
  const safe = getSafeAreaPadding({ width, height });

  const at = (seconds: number) => (seconds * fps) / speed;
  const ease = (from: number, to: number, easing = EASING.enter) =>
    interpolate(frame, [at(from), at(to)], [0, 1], { easing, ...clamp });

  const portrait = height > width;
  const u = portrait ? Math.min(width / 496, height / 896) : Math.min(width / 1024, height / 576);

  const cardIn = spring({
    frame: frame - at(T.card),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.85 },
  });
  const open = ease(T.card, T.card + T.cardFor, EASING.editorial);
  const versionIn = ease(T.version, T.version + T.versionFor);
  const ruleIn = ease(T.rule, T.rule + T.ruleFor, EASING.editorial);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...clamp,
        });

  const cardW = Math.min(width - safe.paddingLeft - safe.paddingRight, cardWidth * u);
  const pad = 30 * u;
  const tagW = 74 * u;

  const colorFor = (kind: ChangeKind) => kindColors[kind] || DEFAULT_KIND_COLORS[kind];

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
          width: cardW,
          borderRadius: 20 * u,
          background: palette.window,
          border: `1px solid ${palette.border}`,
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${24 * u}px ${
            60 * u
          }px ${palette.shadow}`,
          padding: pad,
          overflow: "hidden",
          clipPath: `inset(0 0 ${(1 - open) * 100}% 0 round ${20 * u}px)`,
          opacity: 1 - exit,
          translate: `0 ${(1 - cardIn) * 22 * u + exit * 30 * u}px`,
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "baseline",
            gap: 12 * u,
            opacity: versionIn,
            translate: `0 ${(1 - versionIn) * 12 * u}px`,
          }}
        >
          <span
            style={{
              color: palette.fg,
              fontSize: 38 * u,
              fontWeight: 800,
              letterSpacing: "-0.03em",
              fontVariantNumeric: "tabular-nums",
            }}
          >
            {version}
          </span>
          {date ? (
            <span
              style={{
                color: palette.dim,
                fontSize: 15 * u,
                fontWeight: 500,
              }}
            >
              {date}
            </span>
          ) : null}
        </div>

        {summary ? (
          <div
            style={{
              marginTop: 6 * u,
              color: palette.dim,
              fontSize: 16.5 * u,
              fontWeight: 500,
              opacity: interpolate(versionIn, [0.4, 1], [0, 1], clamp),
            }}
          >
            {summary}
          </div>
        ) : null}

        <div
          style={{
            marginTop: 18 * u,
            marginBottom: 18 * u,
            height: 1,
            width: `${ruleIn * 100}%`,
            background: palette.border,
          }}
        />

        <div style={{ display: "flex", flexDirection: "column", gap: 12 * u }}>
          {changes.map((change, index) => {
            const rowAt = startAtSeconds + index * staggerSeconds;
            const rowIn = ease(rowAt, rowAt + 0.4);
            const textIn = ease(rowAt + 0.08, rowAt + 0.46);
            const color = colorFor(change.kind);

            return (
              <div
                key={index}
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: 14 * u,
                  opacity: rowIn,
                }}
              >
                <span
                  style={{
                    width: tagW,
                    flexShrink: 0,
                    textAlign: "center",
                    padding: `${4 * u}px 0`,
                    borderRadius: 7 * u,
                    background: `color-mix(in srgb, ${color} 12%, transparent)`,
                    border: `1px solid color-mix(in srgb, ${color} 33%, transparent)`,
                    color,
                    fontSize: 14.5 * u,
                    fontWeight: 700,
                    letterSpacing: "0.08em",
                    textTransform: "uppercase",
                    scale: `${0.9 + rowIn * 0.1}`,
                  }}
                >
                  {tags[change.kind]}
                </span>
                <span
                  style={{
                    color: palette.fg,
                    fontSize: 17 * u,
                    fontWeight: 500,
                    opacity: textIn,
                    translate: `${(1 - textIn) * 10 * u}px`,
                  }}
                >
                  {change.text}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}

export function ChangelogEntry({
  kindAdded,
  kindFixed,
  kindChanged,
  kindRemoved,
  addedLabel,
  fixedLabel,
  changedLabel,
  removedLabel,
  ...settings
}: ElementSettings["changelog-entry"]) {
  return (
    <ChangelogEntryView
      {...settings}
      kindColors={{
        added: kindAdded,
        fixed: kindFixed,
        changed: kindChanged,
        removed: kindRemoved,
      }}
      tags={{ added: addedLabel, fixed: fixedLabel, changed: changedLabel, removed: removedLabel }}
    />
  );
}
