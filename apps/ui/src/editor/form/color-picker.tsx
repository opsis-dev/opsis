import { Slider as SliderPrimitive } from "@base-ui/react/slider";
import {
  COLOR_FORMATS,
  formatColor,
  formatOf,
  hueColor,
  opaqueHex,
  parseColor,
  sameColor,
  withChannel,
  type ColorFormat,
  type Hsva,
} from "@opsis/video/common/color";
import { percentText } from "@opsis/video/common/format";
import { isOneOf } from "@opsis/video/common/guards";
import { cn } from "cn";
import { Pipette } from "lucide-react";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type PointerEvent,
  type ReactNode,
} from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { fractionIn } from "@/lib/dom";
import { useCommitDraft } from "@/lib/use-commit-draft";
import { usePointerDrag } from "@/lib/use-pointer-drag";
import { workflowEdits } from "@/store/workflow";

export const CHECKER: CSSProperties = {
  backgroundImage:
    "repeating-conic-gradient(var(--color-muted) 0% 25%, var(--color-background) 0% 50%)",
  backgroundSize: "8px 8px",
};

const FORMAT_LABELS: Record<ColorFormat, string> = {
  hex: "Hex",
  rgb: "RGB",
  hsl: "HSL",
  oklch: "OKLCH",
};

const NO_COLOR: Hsva = { h: 0, s: 0, v: 0, a: 1 };

export function ColorChip({ value, className }: { value: string; className?: string }) {
  const empty = !parseColor(value);
  return (
    <span
      aria-hidden
      className={cn("relative block size-5 overflow-hidden rounded border", className)}
      style={CHECKER}
    >
      <span
        className={cn("absolute inset-0", empty && "bg-transparent")}
        style={empty ? undefined : { background: value }}
      />
    </span>
  );
}

function useGestureWhileOpen() {
  const open = useRef(false);
  const toggle = useCallback((next: boolean) => {
    if (next === open.current) return;
    open.current = next;
    if (next) workflowEdits.beginGesture();
    else workflowEdits.endGesture();
  }, []);
  useEffect(() => () => toggle(false), [toggle]);
  return toggle;
}

export function ColorPopover({
  value,
  placeholder,
  onChange,
  swatches,
  label,
  trigger,
  footer,
}: {
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
  swatches: string[];
  label: string;
  trigger: ReactNode;
  footer?: ReactNode;
}) {
  const gesture = useGestureWhileOpen();
  return (
    <Popover onOpenChange={gesture}>
      <PopoverTrigger
        render={
          <button
            type="button"
            aria-label={`Pick ${label.toLowerCase()}`}
            className="cursor-pointer rounded focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none"
          />
        }
      >
        {trigger}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-64 gap-3">
        <ColorPanel value={value || placeholder || ""} onChange={onChange} swatches={swatches} />
        {footer}
      </PopoverContent>
    </Popover>
  );
}

function useHsva(value: string) {
  const [state, setState] = useState(() => ({ value, color: parseColor(value) ?? NO_COLOR }));
  if (!sameColor(state.value, value)) {
    const color = parseColor(value) ?? NO_COLOR;
    setState({ value, color });
    return { color, setLocal: setState };
  }
  return { color: state.color, setLocal: setState };
}

function ColorPanel({
  value,
  onChange,
  swatches,
}: {
  value: string;
  onChange: (value: string) => void;
  swatches: string[];
}) {
  const { color, setLocal } = useHsva(value);
  const [format, setFormat] = useState<ColorFormat>(() => formatOf(value));
  const pick = (next: Hsva, as: ColorFormat = format) => {
    const text = formatColor(next, as);
    setLocal({ value: text, color: next });
    onChange(text);
  };
  return (
    <div className="flex flex-col gap-3">
      <SaturationArea color={color} onChange={(c) => pick(c)} />
      <ChannelSlider
        label="Hue"
        value={color.h}
        max={360}
        step={1}
        track="linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00)"
        onChange={(h) => pick(withChannel(color, { h }))}
      />
      <ChannelSlider
        label="Opacity"
        value={color.a}
        max={1}
        step={0.01}
        checker
        track={`linear-gradient(to right, transparent, ${opaqueHex(color)})`}
        onChange={(a) => pick(withChannel(color, { a }))}
      />
      <div className="flex items-center gap-2">
        <EyeDropperButton
          onPick={(hex) => {
            const picked = parseColor(hex);
            if (picked) pick({ ...picked, a: color.a });
          }}
        />
        <ColorText value={value} onChange={onChange} />
      </div>
      <ToggleGroup
        variant="outline"
        size="sm"
        value={[format]}
        onValueChange={(v) => {
          const next = v[0];
          if (!isOneOf(COLOR_FORMATS, next)) return;
          setFormat(next);
          if (parseColor(value)) pick(color, next);
        }}
        className="w-full"
      >
        {COLOR_FORMATS.map((f) => (
          <ToggleGroupItem key={f} value={f} className="flex-1 text-xs">
            {FORMAT_LABELS[f]}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {swatches.length ? (
        <div className="flex flex-wrap gap-1.5" role="list" aria-label="Colours in this project">
          {swatches.map((s) => (
            <button
              key={s}
              type="button"
              role="listitem"
              title={s}
              aria-label={`Use ${s}`}
              className="cursor-pointer rounded"
              onClick={() => {
                const picked = parseColor(s);
                if (picked) pick(picked, formatOf(s));
              }}
            >
              <ColorChip value={s} className="size-6" />
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function ColorText({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const draft = useCommitDraft(value, (text) => {
    if (parseColor(text)) onChange(text.trim());
  });
  return (
    <Input
      aria-label="Colour value"
      value={draft.text}
      spellCheck={false}
      onChange={(e) => draft.change(e.target.value)}
      onBlur={draft.commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") draft.commit();
      }}
      aria-invalid={!parseColor(draft.text)}
      className="h-7 font-mono text-xs"
    />
  );
}

function SaturationArea({ color, onChange }: { color: Hsva; onChange: (color: Hsva) => void }) {
  const area = useRef<HTMLDivElement>(null);
  const update = (e: PointerEvent<HTMLDivElement>) => {
    const rect = area.current?.getBoundingClientRect();
    if (!rect) return;
    const { x, y } = fractionIn(rect, e.clientX, e.clientY);
    onChange(withChannel(color, { s: x, v: 1 - y }));
  };
  const drag = usePointerDrag<HTMLDivElement, null>({
    start: (e) => {
      update(e);
      return null;
    },
    move: (e) => update(e),
  });
  const nudge = (ds: number, dv: number) =>
    onChange(withChannel(color, { s: color.s + ds, v: color.v + dv }));
  return (
    <div
      ref={area}
      role="slider"
      tabIndex={0}
      aria-label="Saturation and brightness"
      aria-valuetext={`${percentText(color.s)} saturation, ${percentText(color.v)} brightness`}
      className="relative h-36 w-full cursor-crosshair touch-none rounded-md focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none"
      style={{
        background: `linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, ${hueColor(color.h)})`,
      }}
      onKeyDown={(e) => {
        const step = e.shiftKey ? 0.1 : 0.01;
        const moves: Partial<Record<string, [number, number]>> = {
          ArrowLeft: [-step, 0],
          ArrowRight: [step, 0],
          ArrowUp: [0, step],
          ArrowDown: [0, -step],
        };
        const move = moves[e.key];
        if (!move) return;
        e.preventDefault();
        nudge(...move);
      }}
      {...drag}
    >
      <span
        className="pointer-events-none absolute size-3.5 -translate-1/2 rounded-full border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.4)]"
        style={{ left: `${color.s * 100}%`, top: `${(1 - color.v) * 100}%` }}
      />
    </div>
  );
}

function ChannelSlider({
  label,
  value,
  max,
  step,
  track,
  checker,
  onChange,
}: {
  label: string;
  value: number;
  max: number;
  step: number;
  track: string;
  checker?: boolean;
  onChange: (value: number) => void;
}) {
  return (
    <SliderPrimitive.Root
      aria-label={label}
      value={value}
      min={0}
      max={max}
      step={step}
      thumbAlignment="edge"
      onValueChange={(v) => onChange(typeof v === "number" ? v : (v[0] ?? 0))}
    >
      <SliderPrimitive.Control className="relative flex h-3 w-full touch-none items-center select-none">
        <SliderPrimitive.Track
          className="relative h-3 w-full overflow-hidden rounded-full"
          style={checker ? CHECKER : undefined}
        >
          <span className="absolute inset-0" style={{ background: track }} />
        </SliderPrimitive.Track>
        <SliderPrimitive.Thumb
          aria-label={label}
          className="block size-3.5 rounded-full border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.4)] focus-visible:ring-3 focus-visible:ring-ring/50 focus-visible:outline-none"
        />
      </SliderPrimitive.Control>
    </SliderPrimitive.Root>
  );
}

declare global {
  interface Window {
    EyeDropper?: new () => { open: () => Promise<{ sRGBHex: string }> };
  }
}

function EyeDropperButton({ onPick }: { onPick: (hex: string) => void }) {
  const Api = window.EyeDropper;
  if (!Api) return null;
  return (
    <Button
      variant="outline"
      size="icon-sm"
      aria-label="Pick a colour from the screen"
      title="Pick a colour from the screen"
      onClick={() => {
        new Api()
          .open()
          .then((r) => onPick(r.sRGBHex))
          .catch(() => undefined);
      }}
    >
      <Pipette />
    </Button>
  );
}
