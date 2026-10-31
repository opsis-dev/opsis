import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { useMemo } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

const FIRST_WORD_SHARE = 10 / 13;

function measureWidths(
  words: string[],
  fontSize: number,
  fontWeight: string,
  fontFamily: string,
  letterSpacing: number,
): number[] {
  if (typeof document === "undefined") {
    return words.map((w) => w.length * fontSize * 0.55);
  }
  const ctx = document.createElement("canvas").getContext("2d");
  if (!ctx) return words.map((w) => w.length * fontSize * 0.55);
  ctx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  ctx.letterSpacing = `${letterSpacing * fontSize}px`;
  return words.map((w) => ctx.measureText(w).width);
}

export function KineticCenterBuild({
  fontFamily,
  text,
  entryOffset,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  wordGap,
  wordTime,
  entryBlur,
  letterSpacing,
}: ElementSettings["kinetic-center-build"]) {
  const face = fontOr(fontFamily, "main");
  const frame = useAuthoredFrame() * speed;

  const words = useMemo(() => text.split(" "), [text]);
  const widths = useMemo(
    () => measureWidths(words, fontSize, fontWeight, face, letterSpacing),
    [words, fontSize, fontWeight, face, letterSpacing],
  );

  const gap = wordGap;
  const pushDur = wordTime;
  const firstDur = Math.round(pushDur * FIRST_WORD_SHARE);
  const travelDur = 7;
  const entryScale = 0.992;
  const reflowBlur = 0.8;
  const firstWordY = 6;
  const easing = Easing.bezier(0.2, 0.8, 0.2, 1);

  const positionsAt = useMemo(() => {
    const out: number[][] = [];
    for (let k = 1; k <= words.length; k++) {
      let total = gap * (k - 1);
      for (let j = 0; j < k; j++) total += widths[j];
      let cursor = -total / 2;
      const xs: number[] = [];
      for (let j = 0; j < k; j++) {
        xs.push(cursor + widths[j] / 2);
        cursor += widths[j] + gap;
      }
      out.push(xs);
    }
    return out;
  }, [words, widths, gap]);

  const n = words.length;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        background,
      }}
    >
      <div
        style={{
          position: "relative",
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: face,
          whiteSpace: "nowrap",
        }}
      >
        {words.map((word, j) => {
          const entryStart = j === 0 ? 0 : firstDur + (j - 1) * pushDur;
          const entryEnd = entryStart + (j === 0 ? firstDur : pushDur);
          const targetX = positionsAt[j][j];
          const xFrom = j === 0 ? 0 : targetX + entryOffset;

          let x = targetX;
          let opacity = 1;
          let scale = 1;
          let blur = 0;
          let y = 0;

          if (frame < entryStart) {
            opacity = 0;
            x = xFrom;
            scale = entryScale;
            blur = entryBlur;
            y = j === 0 ? firstWordY : 0;
          } else if (frame <= entryEnd) {
            const range: [number, number] = [entryStart, entryEnd];
            const travelRange: [number, number] = [
              entryStart,
              Math.min(entryStart + travelDur, entryEnd),
            ];
            const opts = { ...CLAMPED, easing };
            x = interpolate(frame, travelRange, [xFrom, targetX], opts);
            opacity = interpolate(frame, range, [0, 1], opts);
            scale = interpolate(frame, range, [entryScale, 1], opts);
            blur = interpolate(frame, range, [entryBlur, 0], opts);
            y = j === 0 ? interpolate(frame, travelRange, [firstWordY, 0], opts) : 0;
          } else {
            for (let w = j + 1; w < n; w++) {
              const pushStart = firstDur + (w - 1) * pushDur;
              const pushEnd = pushStart + pushDur;
              const fromX = positionsAt[w - 1][j];
              const toX = positionsAt[w][j];
              if (frame >= pushEnd) {
                x = toX;
              } else if (frame >= pushStart) {
                x = interpolate(
                  frame,
                  [pushStart, Math.min(pushStart + travelDur, pushEnd)],
                  [fromX, toX],
                  { ...CLAMPED, easing },
                );
                blur = interpolate(
                  frame,
                  [pushStart, (pushStart + pushEnd) / 2, pushEnd],
                  [0, reflowBlur, 0],
                  CLAMPED,
                );
                break;
              } else {
                x = fromX;
                break;
              }
            }
          }

          return (
            <span
              key={j}
              style={{
                position: "absolute",
                left: "50%",
                top: "50%",
                whiteSpace: "nowrap",
                backfaceVisibility: "hidden",
                transform: `translate(-50%, -50%) translate3d(${x}px, ${y}px, 0) scale(${scale})`,
                filter: `blur(${blur}px)`,
                opacity,
              }}
            >
              {word}
            </span>
          );
        })}
      </div>
    </div>
  );
}
