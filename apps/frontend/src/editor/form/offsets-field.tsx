import { FieldLegend, FieldSet } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { formatNumberList, parseNumberList } from "@/lib/number-text";
import { MS_SCALE, parseShown, shownText, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

function withOffset(list: number[], index: number, ms: number): string {
  return formatNumberList(list.map((v, i) => (i === index ? ms : v)));
}

export function OffsetsField({ label, value, onChange, ctx, meta }: FieldProps<string>) {
  const parts = parseNumberList(value);
  const count = Math.max(ctx.layers, parts.length, 1);
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
      <Help meta={meta} />
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
  const input = useDecimalInput(shownText(ms, MS_SCALE), "blur", (text) =>
    onCommit(Math.max(0, parseShown(text, MS_SCALE) ?? 0)),
  );
  return (
    <label className="flex items-center gap-2 text-xs">
      <span className="text-muted-foreground w-14 shrink-0">Layer {layer}</span>
      <Input className="h-7 text-right tabular-nums" {...input} />
      <span className="text-muted-foreground">s</span>
    </label>
  );
}
