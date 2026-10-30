import type Konva from "konva";
import {
  ArrowUpRight,
  Circle as CircleIcon,
  Hash,
  MousePointer2,
  Square,
  Trash2,
  Type,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useRef, useState, type ReactNode, type RefObject } from "react";
import { Arrow, Circle, Ellipse, Group, Label, Layer, Rect, Stage, Tag, Text } from "react-konva";
import { Button } from "@/components/ui/button";
import { FieldDescription } from "@/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { schemaDefaults, type JsonSchema } from "@/editor/form/schema";
import { whenFirstOneOf } from "@/lib/one-of";
import { cn } from "@/lib/utils";
import { WithTooltip } from "../with-tooltip";
import { FieldShell } from "./field-shell";
import {
  drawn,
  kindOf,
  moved,
  pointsOf,
  stepNumbers,
  twoEnded,
  withEnd,
  type MarkKind,
  type MarkPoints,
  type MarkTool,
  type Point,
} from "./marks";
import { SpotFrame } from "./spot-frame";
import type { FormContext, ParamFieldProps, ParamValues } from "./types";

const MAX_MARKS = 50;
const HEAD_LENGTH = 4.5;
const HEAD_WIDTH = 3.2;
const SMALLEST_HEAD = 12;
const HANDLE_RADIUS = 6;

const TOOLS: Array<{ tool: MarkTool; label: string; icon: LucideIcon }> = [
  { tool: "select", label: "Select and move", icon: MousePointer2 },
  { tool: "arrow", label: "Arrow", icon: ArrowUpRight },
  { tool: "rectangle", label: "Box", icon: Square },
  { tool: "oval", label: "Oval", icon: CircleIcon },
  { tool: "step", label: "Step number", icon: Hash },
  { tool: "text", label: "Text", icon: Type },
];

const TOOL_NAMES = TOOLS.map((t) => t.tool);

const KIND_WORDS: Record<MarkKind, string> = {
  arrow: "Arrow",
  rectangle: "Box",
  oval: "Oval",
  step: "Step",
  text: "Text",
};

type Size = { width: number; height: number };

function useElementSize(ref: RefObject<HTMLElement | null>): Size {
  const [size, setSize] = useState<Size>({ width: 0, height: 0 });
  useEffect(() => {
    const element = ref.current;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      if (entry) setSize({ width: entry.contentRect.width, height: entry.contentRect.height });
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [ref]);
  return size;
}

function useNaturalWidth(ctx: FormContext): number | undefined {
  const src = ctx.spotBackground?.still ? ctx.spotBackground.src : undefined;
  const [width, setWidth] = useState<number>();
  useEffect(() => {
    if (!src) return;
    const image = new window.Image();
    function loaded() {
      return setWidth(image.naturalWidth);
    }
    image.addEventListener("load", loaded);
    image.src = src;
    return () => image.removeEventListener("load", loaded);
  }, [src]);
  return src ? width : undefined;
}

function numberIn(mark: ParamValues, key: string, fallback: number): number {
  const v = mark[key];
  return typeof v === "number" ? v : fallback;
}

function textIn(mark: ParamValues, key: string): string {
  const v = mark[key];
  return typeof v === "string" ? v : "";
}

type Drawing = { kind: MarkKind; start: Point; end: Point };

type ShapeProps = {
  mark: ParamValues;
  points: MarkPoints;
  size: Size;
  scale: number;
  step: number | undefined;
};

function MarkShape({ mark, points, size, scale, step }: ShapeProps) {
  const kind = kindOf(mark);
  const colour = textIn(mark, "color") || "#ef4444";
  const line = numberIn(mark, "lineWidth", 6);
  const textSize = numberIn(mark, "size", 32) * scale;
  const font = textIn(mark, "font") || "sans-serif";
  const bold = numberIn(mark, "weight", 700) >= 600 ? "bold" : "normal";
  const head = numberIn(mark, "headSize", 1);
  const fill = numberIn(mark, "fill", 0);
  const radius = numberIn(mark, "radius", 0) * scale;
  const [x1, y1, x2, y2] = [
    points.x * size.width,
    points.y * size.height,
    points.x2 * size.width,
    points.y2 * size.height,
  ];
  switch (kind) {
    case "arrow":
      return (
        <Arrow
          points={[x1, y1, x2, y2]}
          stroke={colour}
          fill={colour}
          strokeWidth={line * scale}
          pointerLength={Math.max(line * HEAD_LENGTH, SMALLEST_HEAD) * head * scale}
          pointerWidth={Math.max(line * HEAD_WIDTH, SMALLEST_HEAD * 0.7) * head * scale}
          hitStrokeWidth={Math.max(line * scale, 12)}
        />
      );
    case "rectangle": {
      const box = {
        x: Math.min(x1, x2),
        y: Math.min(y1, y2),
        width: Math.abs(x2 - x1),
        height: Math.abs(y2 - y1),
        cornerRadius: radius,
      };
      return (
        <Group>
          {fill > 0 ? <Rect {...box} fill={colour} opacity={fill} listening={false} /> : null}
          <Rect
            {...box}
            stroke={colour}
            strokeWidth={line * scale}
            hitStrokeWidth={Math.max(line * scale, 12)}
          />
        </Group>
      );
    }
    case "oval": {
      const oval = {
        x: (x1 + x2) / 2,
        y: (y1 + y2) / 2,
        radiusX: Math.abs(x2 - x1) / 2,
        radiusY: Math.abs(y2 - y1) / 2,
      };
      return (
        <Group>
          {fill > 0 ? <Ellipse {...oval} fill={colour} opacity={fill} listening={false} /> : null}
          <Ellipse
            {...oval}
            stroke={colour}
            strokeWidth={line * scale}
            hitStrokeWidth={Math.max(line * scale, 12)}
          />
        </Group>
      );
    }
    case "step":
      return (
        <Group>
          <Circle x={x1} y={y1} radius={textSize / 2} fill={colour} />
          <Text
            x={x1 - textSize / 2}
            y={y1 - textSize / 2}
            width={textSize}
            height={textSize}
            align="center"
            verticalAlign="middle"
            text={String(step ?? "")}
            fontSize={textSize * 0.6}
            fontFamily={font}
            fontStyle={bold}
            fill={textIn(mark, "textColor") || "#ffffff"}
          />
        </Group>
      );
    case "text": {
      const background = textIn(mark, "textBackground");
      const pad = background ? textSize * 0.3 : 0;
      return (
        <Label x={x1 - pad} y={y1 - pad}>
          <Tag fill={background || undefined} cornerRadius={background ? textSize * 0.25 : 0} />
          <Text
            text={textIn(mark, "text") || "Text"}
            fontSize={textSize}
            fontFamily={font}
            fontStyle={bold}
            fill={colour}
            padding={pad}
          />
        </Label>
      );
    }
  }
}

function EndHandle({
  at,
  size,
  onMove,
  onDone,
}: {
  at: Point;
  size: Size;
  onMove: (to: Point) => void;
  onDone: (to: Point) => void;
}) {
  function where(e: Konva.KonvaEventObject<DragEvent>): Point {
    return {
      x: e.target.x() / size.width,
      y: e.target.y() / size.height,
    };
  }
  return (
    <Circle
      x={at.x * size.width}
      y={at.y * size.height}
      radius={HANDLE_RADIUS}
      fill="#ffffff"
      stroke="#18181b"
      strokeWidth={1.5}
      draggable
      onDragMove={(e) => onMove(where(e))}
      onDragEnd={(e) => onDone(where(e))}
    />
  );
}

function ToolBar({ tool, onTool }: { tool: MarkTool; onTool: (tool: MarkTool) => void }) {
  return (
    <ToggleGroup
      variant="outline"
      size="sm"
      value={[tool]}
      onValueChange={whenFirstOneOf(TOOL_NAMES, onTool)}
    >
      {TOOLS.map(({ tool: name, label, icon: Icon }) => (
        <WithTooltip
          key={name}
          trigger={<ToggleGroupItem value={name} aria-label={label} />}
          tip={label}
        >
          <Icon />
        </WithTooltip>
      ))}
    </ToggleGroup>
  );
}

export function MarksField({
  label,
  meta,
  value: marks,
  onChange,
  ctx,
  all,
  item,
  renderMark,
}: ParamFieldProps<ParamValues[]> & {
  item: JsonSchema;
  renderMark: (mark: ParamValues, onPatch: (patch: ParamValues) => void) => ReactNode;
}) {
  const [tool, setTool] = useState<MarkTool>("arrow");
  const [selected, setSelected] = useState<number | null>(null);
  const [drawing, setDrawing] = useState<Drawing | null>(null);
  const [dragged, setDragged] = useState<{ index: number; points: MarkPoints } | null>(null);
  const frame = useRef<HTMLDivElement>(null);
  const size = useElementSize(frame);
  const natural = useNaturalWidth(ctx);
  const scale = natural && size.width ? size.width / natural : 1;
  const firstStep = typeof all.firstStep === "number" ? all.firstStep : 1;
  const steps = stepNumbers(marks, firstStep);
  const full = marks.length >= MAX_MARKS;
  const current = selected !== null && selected < marks.length ? selected : null;

  function pointer(e: Konva.KonvaEventObject<PointerEvent>): Point | null {
    const at = e.target.getStage()?.getPointerPosition();
    return at && size.width && size.height ? { x: at.x / size.width, y: at.y / size.height } : null;
  }
  function update(index: number, patch: ParamValues) {
    return onChange(marks.map((m, i) => (i === index ? { ...m, ...patch } : m)));
  }
  function remove(index: number) {
    onChange(marks.filter((_, i) => i !== index));
    setSelected(null);
  }

  function begin(e: Konva.KonvaEventObject<PointerEvent>) {
    const at = pointer(e);
    if (!at) return;
    if (tool === "select") {
      if (e.target === e.target.getStage()) setSelected(null);
      return;
    }
    if (!full) setDrawing({ kind: tool, start: at, end: at });
  }
  function follow(e: Konva.KonvaEventObject<PointerEvent>) {
    const at = pointer(e);
    if (drawing && at) setDrawing({ ...drawing, end: at });
  }
  function finish() {
    if (!drawing) return;
    const points = drawn(drawing.kind, drawing.start, drawing.end);
    setDrawing(null);
    if (!points) return;
    const mark = {
      ...schemaDefaults(item),
      kind: drawing.kind,
      ...points,
      ...(drawing.kind === "text" ? { text: "Text" } : {}),
    };
    onChange([...marks, mark]);
    setSelected(marks.length);
  }

  function shownPoints(mark: ParamValues, index: number): MarkPoints {
    return dragged?.index === index ? dragged.points : pointsOf(mark);
  }

  return (
    <FieldShell label={label} meta={meta}>
      <ToolBar tool={tool} onTool={setTool} />
      <SpotFrame
        ref={frame}
        ctx={ctx}
        tabIndex={0}
        className={cn(
          "touch-none select-none outline-none focus-visible:ring-2",
          tool === "select" ? "cursor-default" : "cursor-crosshair",
        )}
        onKeyDown={(e) => {
          if (current !== null && (e.key === "Delete" || e.key === "Backspace")) {
            e.preventDefault();
            remove(current);
          }
        }}
      >
        {size.width > 0 ? (
          <Stage
            width={size.width}
            height={size.height}
            className="absolute inset-0"
            onPointerDown={begin}
            onPointerMove={follow}
            onPointerUp={finish}
          >
            <Layer>
              {marks.map((mark, i) => (
                <Group
                  key={i}
                  draggable={tool === "select"}
                  onPointerDown={() => {
                    if (tool === "select") setSelected(i);
                  }}
                  onDragEnd={(e) => {
                    const dx = e.target.x() / size.width;
                    const dy = e.target.y() / size.height;
                    e.target.position({ x: 0, y: 0 });
                    update(i, moved(pointsOf(mark), dx, dy));
                  }}
                >
                  <MarkShape
                    mark={mark}
                    points={shownPoints(mark, i)}
                    size={size}
                    scale={scale}
                    step={steps[i]}
                  />
                </Group>
              ))}
              {drawing ? (
                <MarkShape
                  mark={{ ...schemaDefaults(item), kind: drawing.kind }}
                  points={{
                    x: drawing.start.x,
                    y: drawing.start.y,
                    x2: drawing.end.x,
                    y2: drawing.end.y,
                  }}
                  size={size}
                  scale={scale}
                  step={firstStep + steps.filter((s) => s !== undefined).length}
                />
              ) : null}
              {current !== null && tool === "select" && twoEnded(kindOf(marks[current] ?? {}))
                ? ([0, 1] as const).map((end) => {
                    const points = shownPoints(marks[current] ?? {}, current);
                    return (
                      <EndHandle
                        key={end}
                        at={
                          end === 0 ? { x: points.x, y: points.y } : { x: points.x2, y: points.y2 }
                        }
                        size={size}
                        onMove={(to) =>
                          setDragged({ index: current, points: withEnd(points, end, to) })
                        }
                        onDone={(to) => {
                          setDragged(null);
                          update(current, withEnd(pointsOf(marks[current] ?? {}), end, to));
                        }}
                      />
                    );
                  })
                : null}
            </Layer>
          </Stage>
        ) : null}
      </SpotFrame>
      <div className="flex flex-wrap gap-1">
        {marks.map((mark, i) => (
          <Button
            key={i}
            variant={i === current ? "secondary" : "ghost"}
            size="sm"
            onClick={() => {
              setSelected(i);
              setTool("select");
            }}
          >
            {i + 1}. {KIND_WORDS[kindOf(mark)]}
            {steps[i] === undefined ? "" : ` ${steps[i]}`}
          </Button>
        ))}
      </div>
      {current !== null && marks[current] ? (
        <div className="bg-muted/40 flex flex-col gap-3 rounded-lg border p-3">
          {renderMark(marks[current], (patch) => update(current, patch))}
          <Button variant="outline" size="sm" className="w-fit" onClick={() => remove(current)}>
            <Trash2 data-icon="inline-start" />
            Remove
          </Button>
        </div>
      ) : null}
      <FieldDescription>
        {full
          ? `Up to ${MAX_MARKS} marks. Remove one to add another.`
          : marks.length
            ? "Select a mark to change it, drag it to move it, or press Delete to remove it."
            : "Pick a kind above and drag on the picture."}
      </FieldDescription>
    </FieldShell>
  );
}
