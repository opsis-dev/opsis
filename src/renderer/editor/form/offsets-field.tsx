import { FieldLegend, FieldSet } from "@/renderer/components/ui/field";
import { Input } from "@/renderer/components/ui/input";
import { formatNumberList, parseNumberList, parseNumberText } from "@/shared/format";
import { trimNumber, unitScale, useDecimalInput } from "./decimal";
import type { FieldProps } from "./types";

const MS = unitScale("ms", 0);

const withOffset = (list: number[], index: number, ms: number): string =>
  formatNumberList(list.map((v, i) => (i === index ? ms : v)));

export function OffsetsField({ label, value, onChange, ctx }: FieldProps<string>) {
  const parts = parseNumberList(value);
  const count = Math.max(ctx.layers ?? 2, parts.length, 1);
  const list = Array.from({ length: count }, (_, i) => parts[i] ?? 0);
  return (
    <FieldSet>
      <FieldLegend variant="label">{label}</FieldLegend>
      <div className="grid grid-cols-2 gap-2">
        {list.map((ms, i) => (
          <OffsetInput
            key={i}
            layer={i + 1}
            ms={ms}
            onCommit={(next) => onChange(withOffset(list, i, next))}
          />
        ))}
      </div>
    </FieldSet>
  );
}

function OffsetInput({
  layer,
  ms,
  onCommit,
}: {
  layer: number;
  ms: number;
  onCommit: (ms: number) => void;
}) {
  const input = useDecimalInput(trimNumber(ms / MS.factor), {
    onCommit: (text) => {
      const n = parseNumberText(text);
      onCommit(n === null ? 0 : Math.max(0, Math.round(n * MS.factor)));
    },
  });
  return (
    <label className="flex items-center gap-2 text-xs">
      <span className="text-muted-foreground w-14 shrink-0">Layer {layer}</span>
      <Input className="h-7 text-right tabular-nums" {...input} />
      <span className="text-muted-foreground">{MS.unit}</span>
    </label>
  );
}
