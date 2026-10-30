import { useMemo } from "react";
import { spring } from "remotion";
import { fontOr } from "#video/common/fonts.ts";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

const COLUMN_EM = 0.7;

function columnWidths(
  from: string,
  to: string,
  fontSize: number,
  fontWeight: string,
  fontFamily: string,
  letterSpacing: number,
): number[] {
  function fixed() {
    return Array.from(from, () => COLUMN_EM * fontSize);
  }
  if (typeof document === "undefined") return fixed();
  const ctx = document.createElement("canvas").getContext("2d");
  if (!ctx) return fixed();
  ctx.font = `${fontWeight} ${fontSize}px ${fontFamily}`;
  ctx.letterSpacing = `${letterSpacing * fontSize}px`;
  return Array.from(from, (char, i) =>
    Math.max(ctx.measureText(char).width, ctx.measureText(to[i] ?? "").width),
  );
}

export function SlotMachineRoll({
  fontFamily,
  from,
  to,
  fontSize,
  color,
  fontWeight,
  speed,
  background,
  columnStagger,
  direction,
  letterSpacing,
}: ElementSettings["slot-machine-roll"]) {
  const face = fontOr(fontFamily, "main");
  const frame = useAuthoredFrame() * speed;
  const { fps } = useAuthoredConfig();

  const len = Math.max(from.length, to.length);
  const paddedFrom = from.padStart(len, " ");
  const paddedTo = to.padStart(len, " ");
  const widths = useMemo(
    () => columnWidths(paddedFrom, paddedTo, fontSize, fontWeight, face, letterSpacing),
    [paddedFrom, paddedTo, fontSize, fontWeight, face, letterSpacing],
  );

  const columns = [];
  for (let i = 0; i < len; i++) {
    const raw = spring({
      frame: frame - i * columnStagger,
      fps,
      config: { damping: 14 },
    });
    const t = Math.max(0, Math.min(1, raw));
    const up = direction === "up";
    const first = up ? paddedFrom[i] : paddedTo[i];
    const second = up ? paddedTo[i] : paddedFrom[i];
    columns.push(
      <span
        key={i}
        style={{
          display: "inline-block",
          overflow: "hidden",
          height: "1.1em",
          lineHeight: "1.1em",
          verticalAlign: "top",
          width: widths[i],
          textAlign: "center",
        }}
      >
        <span
          style={{
            display: "flex",
            flexDirection: "column",
            translate: `0 ${-(up ? t : 1 - t) * fontSize * 1.1}px`,
          }}
        >
          <span style={{ height: "1.1em", whiteSpace: "pre" }}>{first}</span>
          <span style={{ height: "1.1em", whiteSpace: "pre" }}>{second}</span>
        </span>
      </span>,
    );
  }

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
      <span
        style={{
          fontSize,
          fontWeight,
          color,
          letterSpacing: `${letterSpacing}em`,
          fontFamily: face,
          lineHeight: 1.1,
        }}
      >
        {columns}
      </span>
    </div>
  );
}
