import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { spring } from "@opsis/runtime/spring.ts";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";
import { ticksFromWords } from "#video/lib/settings/shape.ts";

type MatrixCell = boolean | string;

type MatrixRow = {
  label: string;
  cells: MatrixCell[];
};

const T = {
  card: 0,
  cardFor: 0.46,
  head: 0.2,
  headFor: 0.4,
  exitFor: 0.42,
} as const;

function Tick({ color, size, progress }: { color: string; size: number; progress: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M5 12.5 L10 17.5 L19 7"
        stroke={color}
        strokeWidth={2.6}
        strokeLinecap="round"
        strokeLinejoin="round"
        pathLength={1}
        strokeDasharray={1}
        strokeDashoffset={1 - progress}
      />
    </svg>
  );
}

function Cross({ color, size, progress }: { color: string; size: number; progress: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M7.5 7.5 L16.5 16.5 M16.5 7.5 L7.5 16.5"
        stroke={color}
        strokeWidth={2.4}
        strokeLinecap="round"
        pathLength={1}
        strokeDasharray={1}
        strokeDashoffset={1 - progress}
      />
    </svg>
  );
}

function ComparisonTableView({
  columns,
  rows,
  title,
  highlight,
  highlightColumn: column,
  highlightLabel,
  startAtSeconds,
  rowStaggerSeconds,
  holdSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  tickColor,
  labelWidth,
  cardWidth,
  fontFamily,
  uppercaseHeadings,
}: Omit<ElementSettings["comparison-table"], "rows"> & {
  rows: MatrixRow[];
  holdSeconds?: number;
}) {
  const highlightColumn = highlight ? column : -1;
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
  const designU = portrait
    ? Math.min(width / 496, height / 896)
    : Math.min(width / 1024, height / 576);
  const tableUnits = 24 * 2 + (title ? 44 : 0) + 44 + 46 * rows.length;
  const u = Math.min(designU, (height - safe.paddingTop - safe.paddingBottom) / tableUnits);

  const cardIn = spring({
    frame: frame - at(T.card),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.85 },
  });
  const open = ease(T.card, T.card + T.cardFor, EASING.editorial);
  const headIn = ease(T.head, T.head + T.headFor);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  const cardW = Math.min(width - safe.paddingLeft - safe.paddingRight, cardWidth * u);
  const pad = 24 * u;
  const labelW = ((cardW - pad * 2) * labelWidth) / 100;
  const colW = (cardW - pad * 2 - labelW) / Math.max(1, columns.length);
  const rowH = 46 * u;
  const headH = 44 * u;

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
          boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${26 * u}px ${
            64 * u
          }px ${palette.shadow}`,
          padding: pad,
          position: "relative",
          overflow: "hidden",
          clipPath: `inset(0 0 ${(1 - open) * 100}% 0 round ${20 * u}px)`,
          opacity: 1 - exit,
          translate: `0 ${(1 - cardIn) * 22 * u + exit * 30 * u}px`,
        }}
      >
        {highlightColumn >= 0 && highlightColumn < columns.length ? (
          <div
            style={{
              position: "absolute",
              left: pad + labelW + highlightColumn * colW,
              top: pad + (title ? 44 * u : 0),
              width: colW,
              height: headH + rows.length * rowH,
              borderRadius: 12 * u,
              background: `color-mix(in srgb, ${accentColor} 7%, transparent)`,
              border: `1px solid color-mix(in srgb, ${accentColor} 23%, transparent)`,
              opacity: headIn,
            }}
          />
        ) : null}

        {title ? (
          <div
            style={{
              height: 44 * u,
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              color: palette.fg,
              fontSize: 26 * u,
              fontWeight: 700,
              letterSpacing: "-0.02em",
              opacity: headIn,
            }}
          >
            <span>{title}</span>
            {highlightLabel && highlightColumn >= 0 ? (
              <span
                style={{
                  padding: `${4 * u}px ${10 * u}px`,
                  borderRadius: 999,
                  background: `color-mix(in srgb, ${accentColor} 13%, transparent)`,
                  border: `1px solid color-mix(in srgb, ${accentColor} 40%, transparent)`,
                  color: accentColor,
                  fontSize: 14.5 * u,
                  fontWeight: 600,
                  letterSpacing: "0.02em",
                }}
              >
                {highlightLabel}
              </span>
            ) : null}
          </div>
        ) : null}

        <div style={{ position: "relative" }}>
          <div style={{ display: "flex", height: headH, alignItems: "flex-end" }}>
            <div style={{ width: labelW }} />
            {columns.map((name, columnIndex) => (
              <div
                key={columnIndex}
                style={{
                  width: colW,
                  textAlign: "center",
                  paddingBottom: 10 * u,
                  color: columnIndex === highlightColumn ? accentColor : palette.dim,
                  fontSize: 15 * u,
                  fontWeight: 600,
                  letterSpacing: "0.04em",
                  textTransform: uppercaseHeadings ? "uppercase" : undefined,
                  opacity: headIn,
                  translate: `0 ${(1 - headIn) * 8 * u}px`,
                }}
              >
                {name}
              </div>
            ))}
          </div>

          {rows.map((row, rowIndex) => {
            const rowAt = startAtSeconds + rowIndex * rowStaggerSeconds;
            const rowIn = ease(rowAt, rowAt + 0.4);
            return (
              <div
                key={rowIndex}
                style={{
                  display: "flex",
                  alignItems: "center",
                  height: rowH,
                  borderTop: `1px solid ${palette.border}`,
                  opacity: rowIn,
                }}
              >
                <div
                  style={{
                    width: labelW,
                    color: palette.fg,
                    fontSize: 16.5 * u,
                    fontWeight: 500,
                    translate: `${(1 - rowIn) * -12 * u}px`,
                  }}
                >
                  {row.label}
                </div>
                {columns.map((_, cellIndex) => {
                  const cell = row.cells[cellIndex] ?? "";
                  const cellAt = rowAt + 0.09 + cellIndex * 0.07;
                  const cellIn = ease(cellAt, cellAt + 0.3, EASING.editorial);
                  const lit = cellIndex === highlightColumn;
                  return (
                    <div
                      key={cellIndex}
                      style={{
                        width: colW,
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        opacity: cellIn,
                      }}
                    >
                      {typeof cell === "string" ? (
                        <span
                          style={{
                            color: lit ? accentColor : palette.fg,
                            fontSize: 16 * u,
                            fontWeight: lit ? 700 : 600,
                            fontVariantNumeric: "tabular-nums",
                            scale: `${0.92 + cellIn * 0.08}`,
                          }}
                        >
                          {cell}
                        </span>
                      ) : cell ? (
                        <Tick
                          color={lit ? accentColor : tickColor}
                          size={22 * u}
                          progress={cellIn}
                        />
                      ) : (
                        <Cross color={palette.faint} size={22 * u} progress={cellIn} />
                      )}
                    </div>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </AbsoluteFill>
  );
}

export function ComparisonTable({ rows, ...settings }: ElementSettings["comparison-table"]) {
  return <ComparisonTableView {...settings} rows={ticksFromWords(rows)} />;
}
