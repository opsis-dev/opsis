import { type CSSProperties, type ReactNode, useId } from "react";
import type { TerminalLook } from "#video/generated/spec.ts";
import {
  CursorShape,
  hexColour,
  Redaction,
  RunFlag,
  type Run,
  type TerminalFrame,
  type TerminalFrames,
} from "#video/terminal/frames.ts";
import { spriteFor, strokeFor, type SpriteShape } from "#video/terminal/sprites.ts";

export const TERMINAL_FONTS = [
  "Opsis Terminal",
  "Opsis Terminal Symbols",
  "Opsis Terminal Symbols 2",
  "Opsis Terminal CJK",
  "Opsis Terminal Emoji",
] as const;

export const TERMINAL_FONT_STACK = `${TERMINAL_FONTS.map((f) => `"${f}"`).join(", ")}, monospace`;

const CELL_WIDTH_EM = 0.6;
const BLUR_SHARE = 0.35;
const PIXEL_SHADES = [0.55, 0.7, 0.45, 0.62];
const PIXELS_PER_CELL = 2;
const WHITE = 0xffffff;

export type ViewLook = Pick<
  TerminalLook,
  "fontSize" | "lineHeight" | "letterSpacing" | "padding" | "cursor"
>;

export type CellBox = { width: number; height: number };

export type Measure = (text: string, bold: boolean) => number | null;

const widths = new Map<string, number>();
let canvas: CanvasRenderingContext2D | null = null;

export function measurer(fontSize: number): Measure {
  return (text, bold) => {
    const key = `${bold ? 7 : 4}|${fontSize}|${text}`;
    const known = widths.get(key);
    if (known !== undefined) return known;
    canvas ??= document.createElement("canvas").getContext("2d");
    if (!canvas) return null;
    canvas.font = `${bold ? 700 : 400} ${fontSize}px ${TERMINAL_FONT_STACK}`;
    const width = canvas.measureText(text).width;
    widths.set(key, width);
    return width;
  };
}

export function cellBox(look: ViewLook): CellBox {
  return {
    width: look.fontSize * CELL_WIDTH_EM + look.letterSpacing,
    height: Math.round(look.fontSize * look.lineHeight),
  };
}

export function screenSize(look: ViewLook, cols: number, rows: number): CellBox {
  const cell = cellBox(look);
  return {
    width: Math.round(cols * cell.width + 2 * look.padding),
    height: Math.round(rows * cell.height + 2 * look.padding),
  };
}

function decoration(run: Run): CSSProperties {
  const lines = [
    run.underline ? "underline" : "",
    run.flags & RunFlag.strikethrough ? "line-through" : "",
    run.flags & RunFlag.overline ? "overline" : "",
  ].filter(Boolean);
  if (!lines.length) return {};
  const style = ["solid", "solid", "double", "wavy", "dotted", "dashed"][run.underline] ?? "solid";
  return {
    textDecorationLine: lines.join(" "),
    textDecorationStyle: style as CSSProperties["textDecorationStyle"],
    textDecorationColor: hexColour(run.underlineColour),
  };
}

type Placed = { run: Run; y: number; shapes: SpriteShape[] };

function rectPath(x: number, y: number, width: number, height: number): string {
  return `M${x} ${y}h${width}v${height}h${-width}Z`;
}

function translate(d: string, dx: number, dy: number): string {
  return d.replace(/([MLC])([^MLCZ]+)/g, (_, cmd: string, args: string) => {
    const n = args
      .trim()
      .split(/[\s,]+/)
      .map(Number);
    return `${cmd}${n.map((v, k) => (k % 2 === 0 ? v + dx : v + dy)).join(" ")}`;
  });
}

type Boxed = {
  shapes: SpriteShape[];
  dx: number;
  dy: number;
  width: number;
  height: number;
  colour: string;
};

function shapePath(shape: SpriteShape, dx: number, dy: number): string {
  if (shape.kind === "rect") return rectPath(shape.x + dx, shape.y + dy, shape.width, shape.height);
  if (shape.kind === "inverted") return "";
  return translate(shape.d, dx, dy);
}

function ShapeElement({
  shape,
  dx,
  dy,
  colour,
  inside,
}: {
  shape: SpriteShape;
  dx: number;
  dy: number;
  colour: string;
  inside?: string;
}) {
  if (shape.kind === "stroke" && shape.inner && inside)
    return (
      <>
        <clipPath id={inside}>
          <path d={shapePath(shape, dx, dy)} />
        </clipPath>
        <path
          d={shapePath(shape, dx, dy)}
          fill="none"
          stroke={colour}
          strokeWidth={shape.width * 2}
          clipPath={`url(#${inside})`}
        />
      </>
    );
  if (shape.kind === "stroke")
    return (
      <path d={shapePath(shape, dx, dy)} fill="none" stroke={colour} strokeWidth={shape.width} />
    );
  if (shape.kind === "inverted") return null;
  const opacity = shape.kind === "rect" ? shape.opacity : (shape.opacity ?? 1);
  return <path d={shapePath(shape, dx, dy)} fill={colour} fillOpacity={opacity} />;
}

function BoxedShapes({ id, boxed }: { id: string; boxed: Boxed }) {
  const { dx, dy, width, height, colour } = boxed;
  const cell = <rect x={dx} y={dy} width={width} height={height} />;
  const inverted = boxed.shapes.flatMap((s) => (s.kind === "inverted" ? s.shapes : []));
  const clipped = boxed.shapes.filter((s) => s.kind !== "inverted");
  return (
    <>
      <clipPath id={`${id}-clip`}>{cell}</clipPath>
      {clipped.length ? (
        <g clipPath={`url(#${id}-clip)`}>
          {clipped.map((shape, i) => (
            <ShapeElement
              key={i}
              shape={shape}
              dx={dx}
              dy={dy}
              colour={colour}
              inside={`${id}-inner-${i}`}
            />
          ))}
        </g>
      ) : null}
      {inverted.length ? (
        <>
          <mask
            id={`${id}-mask`}
            maskUnits="userSpaceOnUse"
            x={dx}
            y={dy}
            width={width}
            height={height}
          >
            <rect x={dx} y={dy} width={width} height={height} fill={hexColour(WHITE)} />
            {inverted.map((shape, i) => (
              <ShapeElement key={i} shape={shape} dx={dx} dy={dy} colour={hexColour(0)} />
            ))}
          </mask>
          <rect
            x={dx}
            y={dy}
            width={width}
            height={height}
            fill={colour}
            mask={`url(#${id}-mask)`}
          />
        </>
      ) : null}
    </>
  );
}

function boxedShape(shape: SpriteShape): boolean {
  if (shape.kind === "inverted") return true;
  if (shape.kind === "rect") return false;
  return shape.clip === true || (shape.kind === "stroke" && shape.inner === true);
}

function SpriteLayer({ placed, look, size }: { placed: Placed[]; look: ViewLook; size: CellBox }) {
  const id = useId().replaceAll(":", "");
  const cell = cellBox(look);
  const fills = new Map<string, string[]>();
  const strokes: Array<{ d: string; colour: string; width: number }> = [];
  const boxed: Boxed[] = [];
  for (const { run, y, shapes } of placed) {
    const dx = look.padding + run.col * cell.width;
    const dy = look.padding + y * cell.height;
    const colour = hexColour(run.fg);
    const own = shapes.filter((s) => boxedShape(s));
    if (own.length)
      boxed.push({
        shapes: own,
        dx,
        dy,
        width: cell.width * run.cells,
        height: cell.height,
        colour,
      });
    for (const shape of shapes) {
      if (boxedShape(shape)) continue;
      if (shape.kind === "stroke") {
        strokes.push({ d: translate(shape.d, dx, dy), colour, width: shape.width });
        continue;
      }
      if (shape.kind === "inverted") continue;
      const opacity = shape.kind === "rect" ? shape.opacity : (shape.opacity ?? 1);
      const key = `${colour}|${opacity}`;
      fills.set(key, [...(fills.get(key) ?? []), shapePath(shape, dx, dy)]);
    }
  }
  return (
    <svg
      width={size.width}
      height={size.height}
      viewBox={`0 0 ${size.width} ${size.height}`}
      style={{ position: "absolute", left: 0, top: 0, pointerEvents: "none" }}
    >
      {[...fills].map(([key, parts]) => {
        const [colour, opacity] = key.split("|");
        return <path key={key} d={parts.join("")} fill={colour} fillOpacity={Number(opacity)} />;
      })}
      {strokes.map((s, i) => (
        <path key={i} d={s.d} fill="none" stroke={s.colour} strokeWidth={s.width} />
      ))}
      {boxed.map((b, i) => (
        <BoxedShapes key={i} id={`${id}-${i}`} boxed={b} />
      ))}
    </svg>
  );
}

function sprite(run: Run, cell: CellBox, thickness: number) {
  const chars = [...run.text];
  if (chars.length !== 1) return null;
  const cp = chars[0]?.codePointAt(0) ?? 0;
  return spriteFor(cp, { width: cell.width * run.cells, height: cell.height, thickness });
}

function Masked({ run, box, bg }: { run: Run; box: CellBox; bg: string }) {
  if (run.redaction === Redaction.solid)
    return <div style={{ position: "absolute", inset: 0, background: hexColour(run.fg) }} />;
  if (run.redaction === Redaction.blur)
    return (
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `color-mix(in oklch, ${bg}, currentColor ${BLUR_SHARE * 100}%)`,
          filter: `blur(${box.height / 4}px)`,
        }}
      />
    );
  const blocks: ReactNode[] = [];
  const side = box.height / PIXELS_PER_CELL;
  const across = Math.ceil(box.width / side);
  for (let y = 0; y < PIXELS_PER_CELL; y++)
    for (let x = 0; x < across; x++)
      blocks.push(
        <div
          key={`${x}:${y}`}
          style={{
            position: "absolute",
            left: x * side,
            top: y * side,
            width: side,
            height: side,
            background: bg,
            filter: `brightness(${PIXEL_SHADES[(x + y * 3 + run.col) % PIXEL_SHADES.length]})`,
          }}
        />,
      );
  return <>{blocks}</>;
}

function fitted(run: Run, box: CellBox, measure: Measure | undefined): CSSProperties {
  if (!measure || [...run.text].length !== 1) return {};
  const width = measure(run.text, (run.flags & RunFlag.bold) !== 0);
  if (!width || width <= box.width) return {};
  return { transform: `scaleX(${box.width / width})`, transformOrigin: "center" };
}

function RunView({
  run,
  y,
  look,
  screenBg,
  thickness,
  measure,
}: {
  run: Run;
  y: number;
  look: ViewLook;
  screenBg: number;
  thickness: number;
  measure?: Measure;
}) {
  const cell = cellBox(look);
  const box = { width: cell.width * run.cells, height: cell.height };
  const fg = hexColour(run.fg);
  const place: CSSProperties = {
    position: "absolute",
    left: look.padding + run.col * cell.width,
    top: look.padding + y * cell.height,
    width: box.width,
    height: box.height,
    color: fg,
    ...(run.bg !== screenBg ? { background: hexColour(run.bg) } : {}),
  };
  if (run.redaction !== Redaction.none)
    return (
      <div style={{ ...place, overflow: "hidden" }}>
        <Masked run={run} box={box} bg={hexColour(run.bg)} />
      </div>
    );
  if (sprite(run, cell, thickness))
    return run.bg !== screenBg ? <div style={{ ...place, color: undefined }} /> : null;
  return (
    <div
      style={{
        ...place,
        whiteSpace: "pre",
        lineHeight: `${cell.height}px`,
        letterSpacing: look.letterSpacing,
        fontWeight: run.flags & RunFlag.bold ? 700 : 400,
        fontStyle: run.flags & RunFlag.italic ? "italic" : "normal",
        textAlign: [...run.text].length === 1 ? "center" : "left",
        ...decoration(run),
      }}
    >
      <span style={{ display: "inline-block", ...fitted(run, box, measure) }}>{run.text}</span>
    </div>
  );
}

function shapeOf(look: ViewLook, frame: TerminalFrame): number {
  switch (look.cursor) {
    case "block":
      return CursorShape.block;
    case "bar":
      return CursorShape.bar;
    case "underline":
      return CursorShape.underline;
    case "program":
      return frame.cursor.shape;
  }
}

function charAt(runs: Run[], x: number): Run | undefined {
  return runs.find((r) => x >= r.col && x < r.col + r.cells);
}

function Cursor({
  frame,
  runs,
  look,
  thickness,
}: {
  frame: TerminalFrame;
  runs: Run[];
  look: ViewLook;
  thickness: number;
}) {
  const cell = cellBox(look);
  const { x, y, colour } = frame.cursor;
  const under = charAt(runs, x);
  const wide = under && under.col === x ? under.cells : 1;
  const left = look.padding + x * cell.width;
  const top = look.padding + y * cell.height;
  const width = cell.width * wide;
  const paint = hexColour(colour);
  const shape = shapeOf(look, frame);
  const base: CSSProperties = { position: "absolute", left, top, pointerEvents: "none" };
  if (shape === CursorShape.bar)
    return (
      <div style={{ ...base, width: thickness * 2, height: cell.height, background: paint }} />
    );
  if (shape === CursorShape.underline)
    return (
      <div
        style={{
          ...base,
          top: top + cell.height - thickness * 2,
          width,
          height: thickness * 2,
          background: paint,
        }}
      />
    );
  if (shape === CursorShape.hollow)
    return (
      <div
        style={{
          ...base,
          width,
          height: cell.height,
          boxSizing: "border-box",
          border: `${thickness}px solid ${paint}`,
        }}
      />
    );
  const text = under ? ([...under.text][x - under.col] ?? "") : "";
  return (
    <div
      style={{
        ...base,
        width,
        height: cell.height,
        background: paint,
        color: hexColour(frame.bg),
        whiteSpace: "pre",
        lineHeight: `${cell.height}px`,
        textAlign: "center",
        fontWeight: under && under.flags & RunFlag.bold ? 700 : 400,
      }}
    >
      {text.trim() ? text : null}
    </div>
  );
}

export function frameText(frames: TerminalFrames, frame: TerminalFrame): string {
  const seen = new Set<string>();
  for (const index of frame.rowIndices)
    for (const run of frames.rowTable[index] ?? []) for (const ch of run.text) seen.add(ch);
  return [...seen].join("");
}

export function TerminalView({
  frames,
  frame,
  look,
  cursorOn,
  measure,
}: {
  frames: TerminalFrames;
  frame: TerminalFrame;
  look: ViewLook;
  cursorOn: boolean;
  measure?: Measure;
}) {
  const size = screenSize(look, frame.cols, frame.rows);
  const thickness = strokeFor(look.fontSize);
  const cursorRuns = frames.rowTable[frame.rowIndices[frame.cursor.y] ?? -1] ?? [];
  const cell = cellBox(look);
  const placed: Placed[] = frame.rowIndices.flatMap((index, y) =>
    (frames.rowTable[index] ?? []).flatMap((run) => {
      const shapes = sprite(run, cell, thickness);
      return shapes ? [{ run, y, shapes }] : [];
    }),
  );
  return (
    <div
      style={{
        position: "relative",
        width: size.width,
        height: size.height,
        overflow: "hidden",
        background: hexColour(frame.bg),
        fontFamily: TERMINAL_FONT_STACK,
        fontSize: look.fontSize,
        fontKerning: "none",
        fontVariantLigatures: "none",
      }}
    >
      {frame.rowIndices.flatMap((index, y) =>
        (frames.rowTable[index] ?? []).map((run) => (
          <RunView
            key={`${y}:${run.col}`}
            run={run}
            y={y}
            look={look}
            screenBg={frame.bg}
            thickness={thickness}
            measure={measure}
          />
        )),
      )}
      <SpriteLayer placed={placed} look={look} size={size} />
      {frame.cursor.visible && cursorOn ? (
        <Cursor frame={frame} runs={cursorRuns} look={look} thickness={thickness} />
      ) : null}
    </div>
  );
}
