import { AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { codePalette } from "#video/lib/code/code-syntax.tsx";
import { getSafeAreaPadding } from "#video/lib/layout/layout.ts";
import { EASING } from "#video/lib/motion/motion-tokens.ts";

export type KanbanCard = {
  title: string;
  meta?: string;
  column: number;
  tint?: string;
  moveTo?: number;
  moveAtSeconds?: number;
};

const T = {
  board: 0,
  boardFor: 0.5,
  deal: 0.18,
  dealFor: 0.42,
  exitFor: 0.42,
} as const;

export function KanbanMove({
  columns,
  cards: cardsProp,
  moveSeconds,
  dealStaggerSeconds,
  holdSeconds,
  accentColor,
  backgroundColor,
  theme,
  speed,
  showCounts,
  fontFamily,
}: ElementSettings["kanban-move"] & { holdSeconds?: number }) {
  const frame = useCurrentFrame();
  const { fps, width, height } = useVideoConfig();
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

  const lastColumn = Math.max(0, columns.length - 1);
  function onBoard(column: number) {
    return Math.min(lastColumn, Math.max(0, Math.round(column)));
  }
  const cards: KanbanCard[] = cardsProp.map((card) => {
    const column = onBoard(card.column);
    const moveTo =
      card.moveTo === undefined || card.moveAtSeconds === undefined
        ? undefined
        : onBoard(card.moveTo);
    return { ...card, column, moveTo: moveTo === column ? undefined : moveTo };
  });

  function startSlot(card: KanbanCard, index: number) {
    return cards.filter((other, otherIndex) => other.column === card.column && otherIndex < index)
      .length;
  }

  function landingSlot(card: KanbanCard, index: number) {
    const arrivesAt = card.moveAtSeconds ?? 0;
    const resident = cards.filter((other) => other.column === card.moveTo).length;
    const gone = cards.filter(
      (other) =>
        other.column === card.moveTo &&
        other.moveTo !== undefined &&
        (other.moveAtSeconds ?? 0) < arrivesAt,
    ).length;
    const earlier = cards.filter(
      (other, otherIndex) =>
        otherIndex !== index &&
        other.moveTo === card.moveTo &&
        (other.moveAtSeconds ?? 0) < arrivesAt,
    ).length;
    return resident - gone + earlier;
  }

  const rows = Math.max(
    3,
    ...cards.map(
      (card, index) =>
        Math.max(startSlot(card, index), card.moveTo === undefined ? 0 : landingSlot(card, index)) +
        1,
    ),
  );
  const u = Math.min(
    designU,
    (height - safe.paddingTop - safe.paddingBottom) / (34 + rows * 74 + (rows - 1) * 10),
  );

  const boardIn = spring({
    frame: frame - at(T.board),
    fps,
    config: { damping: 18, stiffness: 130, mass: 0.9 },
  });
  const open = ease(T.board, T.board + T.boardFor, EASING.editorial);

  const exit =
    holdSeconds === undefined
      ? 0
      : interpolate(frame, [at(holdSeconds), at(holdSeconds + T.exitFor)], [0, 1], {
          easing: EASING.exit,
          ...CLAMPED,
        });

  const boardW = Math.min(width - safe.paddingLeft - safe.paddingRight, 1000 * u);
  const gutter = 18 * u;
  const columnW = (boardW - gutter * (columns.length - 1)) / columns.length;
  const cardH = 74 * u;
  const cardGap = 10 * u;
  const headerH = 34 * u;
  const listH = cardH * rows + cardGap * (rows - 1);

  function progressOf(card: KanbanCard) {
    return card.moveTo === undefined || card.moveAtSeconds === undefined
      ? 0
      : ease(card.moveAtSeconds, card.moveAtSeconds + moveSeconds, EASING.editorial);
  }

  function closedUpBy(column: number, ownSlot: number, presentFromSeconds: number, index: number) {
    return cards.reduce((sum, other, otherIndex) => {
      if (otherIndex === index) return sum;
      if (other.column !== column || other.moveTo === undefined) return sum;
      if ((other.moveAtSeconds ?? 0) < presentFromSeconds) return sum;
      if (startSlot(other, otherIndex) >= ownSlot) return sum;
      return sum + progressOf(other);
    }, 0);
  }

  function columnX(columnIndex: number) {
    return columnIndex * (columnW + gutter);
  }
  function slotY(slot: number) {
    return headerH + slot * (cardH + cardGap);
  }

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
          position: "relative",
          width: boardW,
          height: headerH + listH,
          opacity: open * (1 - exit),
          transform: `translateY(${(1 - boardIn) * 22 * u + exit * 28 * u}px)`,
        }}
      >
        {columns.map((column, columnIndex) => {
          const settled = cards.filter((card, index) => {
            const progress = progressOf(card);
            const where = progress > 0.5 ? card.moveTo : card.column;
            return (
              where === columnIndex &&
              ease(
                T.deal + index * dealStaggerSeconds,
                T.deal + index * dealStaggerSeconds + T.dealFor,
              ) > 0.5
            );
          }).length;

          const incoming = cards.reduce((peak, card) => {
            if (card.moveTo !== columnIndex) return peak;
            const progress = progressOf(card);
            return Math.max(peak, Math.min(1, progress * (1 - progress) * 4));
          }, 0);

          return (
            <div
              key={columnIndex}
              style={{
                position: "absolute",
                left: columnX(columnIndex),
                top: 0,
                width: columnW,
                height: headerH + listH,
                borderRadius: 14 * u,
                background: palette.band,
                border: `1px solid ${
                  incoming > 0.02
                    ? `color-mix(in srgb, ${accentColor} ${((40 + incoming * 120) / 2.55).toFixed(1)}%, transparent)`
                    : palette.border
                }`,
                boxShadow:
                  incoming > 0.02
                    ? `0 0 ${incoming * 26 * u}px color-mix(in srgb, ${accentColor} 20%, transparent)`
                    : "none",
              }}
            >
              <div
                style={{
                  height: headerH,
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: `0 ${12 * u}px`,
                }}
              >
                <span
                  style={{
                    color: palette.dim,
                    fontSize: 14.5 * u,
                    fontWeight: 600,
                    letterSpacing: "0.06em",
                    textTransform: "uppercase",
                  }}
                >
                  {column}
                </span>
                <span
                  style={{
                    color: palette.faint,
                    fontSize: 14.5 * u,
                    fontWeight: 600,
                    fontVariantNumeric: "tabular-nums",
                  }}
                >
                  {showCounts ? settled : null}
                </span>
              </div>
            </div>
          );
        })}

        {cards.map((card, index) => {
          const dealAt = T.deal + index * dealStaggerSeconds;
          const dealt = ease(dealAt, dealAt + T.dealFor);
          const progress = progressOf(card);

          const home = startSlot(card, index);
          const fromSlot = home - closedUpBy(card.column, home, Number.NEGATIVE_INFINITY, index);
          const toSlot =
            card.moveTo === undefined
              ? fromSlot
              : landingSlot(card, index) -
                closedUpBy(card.moveTo, landingSlot(card, index), card.moveAtSeconds ?? 0, index);

          const x = interpolate(
            progress,
            [0, 1],
            [columnX(card.column), columnX(card.moveTo ?? card.column)],
          );
          const y = interpolate(progress, [0, 1], [slotY(fromSlot), slotY(toSlot)]);

          const lift = Math.sin(Math.PI * progress);
          const tint = card.tint ?? accentColor;

          return (
            <div
              key={index}
              style={{
                position: "absolute",
                left: x + 8 * u,
                top: y - lift * 22 * u,
                width: columnW - 16 * u,
                height: cardH,
                zIndex: lift > 0.01 ? 20 : 1,
                borderRadius: 11 * u,
                background: palette.window,
                border: `1px solid ${lift > 0.05 ? `color-mix(in srgb, ${tint} 40%, transparent)` : palette.border}`,
                boxShadow: `inset 0 1px 0 ${palette.highlight}, 0 ${
                  (4 + lift * 20) * u
                }px ${(10 + lift * 34) * u}px ${palette.shadow}`,
                opacity: dealt,
                transform: `scale(${(0.94 + dealt * 0.06) * (1 + lift * 0.05)}) rotate(${
                  lift * -1.6
                }deg)`,
                transformOrigin: "center",
                display: "flex",
                alignItems: "center",
                gap: 10 * u,
                padding: `0 ${12 * u}px`,
                overflow: "hidden",
              }}
            >
              <div
                style={{
                  width: 3 * u,
                  height: 40 * u,
                  borderRadius: 999,
                  background: tint,
                  opacity: 0.85,
                  flexShrink: 0,
                }}
              />
              <div style={{ minWidth: 0 }}>
                <div
                  style={{
                    color: palette.fg,
                    fontSize: 16 * u,
                    fontWeight: 600,
                    letterSpacing: "-0.01em",
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {card.title}
                </div>
                {card.meta ? (
                  <div
                    style={{
                      marginTop: 4 * u,
                      color: palette.dim,
                      fontSize: 14.5 * u,
                      fontWeight: 500,
                      whiteSpace: "nowrap",
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                    }}
                  >
                    {card.meta}
                  </div>
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
