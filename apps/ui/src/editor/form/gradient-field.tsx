import { percentText } from "@opsis/video/common/format";
import { clamp } from "@opsis/video/common/math";
import { Trash2 } from "lucide-react";
import { useEffect, useRef, useState, type PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { commands, type GradientEdit, type GradientView } from "@/generated/bindings";
import { fractionIn } from "@/lib/dom";
import { parseNumberText } from "@/lib/number-text";
import { useCommitDraft } from "@/lib/use-commit-draft";
import { usePointerDrag } from "@/lib/use-pointer-drag";
import { workflowEdits } from "@/store/workflow";
import { ColorInput } from "./color-field";
import { CHECKER } from "./color-picker";
import { trimNumber, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

type Edit = (change: GradientEdit) => void;

function StopBar({
  view,
  selected,
  onSelect,
  onEdit,
}: {
  view: GradientView;
  selected: number;
  onSelect: (index: number) => void;
  onEdit: Edit;
}) {
  const bar = useRef<HTMLDivElement>(null);
  const atOf = (e: PointerEvent) => {
    const rect = bar.current?.getBoundingClientRect();
    return rect ? fractionIn(rect, e.clientX, e.clientY).x * 100 : 0;
  };
  const drag = usePointerDrag<HTMLButtonElement, number>({
    start: (e) => {
      const index = Number(e.currentTarget.dataset.stop);
      onSelect(index);
      workflowEdits.beginGesture();
      return index;
    },
    move: (e, index) => onEdit({ op: "stopAt", index, at: atOf(e) }),
    end: (index) => {
      onEdit({ op: "sort", index });
      workflowEdits.endGesture();
    },
  });
  return (
    <div className="relative px-2 pb-3">
      <div
        ref={bar}
        className="h-6 w-full cursor-copy rounded-md border"
        style={CHECKER}
        onPointerDown={(e) => {
          if (e.target !== e.currentTarget || !view.canAdd) return;
          onEdit({ op: "addStop", at: atOf(e) });
        }}
      >
        <div
          className="pointer-events-none h-full w-full rounded-md"
          style={{ background: view.strip }}
        />
      </div>
      {view.gradient.stops.map((stop, i) => (
        <button
          key={i}
          type="button"
          data-stop={i}
          aria-label={`Colour stop ${i + 1} at ${percentText(stop.at / 100)}`}
          aria-pressed={i === selected}
          className="absolute top-4 size-4 -translate-x-1/2 cursor-ew-resize touch-none rounded-sm border-2 border-white shadow-[0_0_0_1px_rgba(0,0,0,0.5)] aria-pressed:ring-2 aria-pressed:ring-ring"
          style={{
            left: `calc(0.5rem + (100% - 1rem) * ${stop.at / 100})`,
            background: stop.color,
          }}
          onKeyDown={(e) => {
            const step = e.key === "ArrowLeft" ? -1 : e.key === "ArrowRight" ? 1 : 0;
            if (!step) return;
            e.preventDefault();
            onEdit({ op: "stopAt", index: i, at: stop.at + step * (e.shiftKey ? 10 : 1) });
          }}
          {...drag}
        />
      ))}
    </div>
  );
}

function SmallNumber({
  label,
  value,
  unit,
  onChange,
}: {
  label: string;
  value: number;
  unit: string;
  onChange: (value: number) => void;
}) {
  const input = useDecimalInput(trimNumber(value), "edit", (text) => {
    const n = parseNumberText(text);
    if (n !== null) onChange(n);
  });
  return (
    <label className="text-muted-foreground flex items-center gap-1 text-xs">
      {label}
      <Input aria-label={label} className="h-7 w-16 text-right tabular-nums" {...input} />
      {unit}
    </label>
  );
}

function RawGradient({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  const draft = useCommitDraft(value, onChange);
  return (
    <Input
      aria-label="Gradient as text"
      value={draft.text}
      spellCheck={false}
      onChange={(e) => draft.change(e.target.value)}
      onBlur={draft.commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") draft.commit();
      }}
      className="h-7 font-mono text-xs"
    />
  );
}

function useGradientView(
  value: string,
  onChange: (css: string) => void,
  onSelect: (i: number) => void,
) {
  const [shown, setShown] = useState<{ css: string; view: GradientView | null } | null>(null);
  const css = useRef(value);
  useEffect(() => {
    css.current = value;
    let live = true;
    void commands.gradientView(value).then((view) => {
      if (live) setShown({ css: value, view });
    });
    return () => {
      live = false;
    };
  }, [value]);
  const edit: Edit = (change) =>
    void commands.editGradient(css.current, change).then((view) => {
      if (!view) return;
      css.current = view.css;
      setShown({ css: view.css, view });
      if (view.selected !== undefined) onSelect(view.selected);
      onChange(view.css);
    });
  return { shown, edit };
}

export function GradientField({ id, label, meta, value, onChange, ctx }: FieldProps<string>) {
  const [selected, setSelected] = useState(0);
  const { shown, edit } = useGradientView(value, onChange, setSelected);
  const view = shown?.view ?? null;
  const index = view ? clamp(selected, 0, view.gradient.stops.length - 1) : 0;
  const stop = view?.gradient.stops[index];
  return (
    <Field>
      <FieldLabel htmlFor={id}>{label}</FieldLabel>
      {view && stop ? (
        <>
          <StopBar view={view} selected={index} onSelect={setSelected} onEdit={edit} />
          <div className="flex flex-wrap items-center gap-2">
            <ToggleGroup
              variant="outline"
              size="sm"
              value={[view.gradient.kind]}
              onValueChange={(v) => {
                if (v[0] === "linear" || v[0] === "radial") edit({ op: "kind", kind: v[0] });
              }}
            >
              <ToggleGroupItem value="linear">Straight</ToggleGroupItem>
              <ToggleGroupItem value="radial">Round</ToggleGroupItem>
            </ToggleGroup>
            {view.gradient.kind === "linear" ? (
              <SmallNumber
                label="Angle"
                unit="°"
                value={view.gradient.angle}
                onChange={(angle) => edit({ op: "angle", angle })}
              />
            ) : null}
          </div>
          <div className="flex items-center gap-2">
            <div className="min-w-0 flex-1">
              <ColorInput
                label={`Stop ${index + 1} colour`}
                value={stop.color}
                swatches={ctx.swatches}
                onChange={(color) => edit({ op: "stopColor", index, color })}
              />
            </div>
            <SmallNumber
              label="At"
              unit="%"
              value={stop.at}
              onChange={(at) => edit({ op: "stopAt", index, at })}
            />
            <Button
              variant="ghost"
              size="icon-sm"
              aria-label="Remove this colour stop"
              disabled={!view.canRemove}
              onClick={() => edit({ op: "removeStop", index })}
            >
              <Trash2 />
            </Button>
          </div>
        </>
      ) : shown ? (
        <p className="text-muted-foreground text-xs">
          This gradient can only be edited as text. Use two or more colours in a straight or round
          gradient to edit it here.
        </p>
      ) : null}
      <RawGradient value={value} onChange={onChange} />
      <Help meta={meta} />
    </Field>
  );
}
