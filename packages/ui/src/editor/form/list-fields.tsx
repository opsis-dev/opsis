import { parseNumberText } from "@opsis/core/text-input";
import { GripVertical, Plus, Trash2, X } from "lucide-react";
import { useRef, type ReactNode } from "react";
import { Button } from "@/components/ui/button";
import { FieldLegend, FieldSet } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Sortable,
  SortableContent,
  SortableItem,
  SortableItemHandle,
} from "@/components/ui/sortable";
import { useItemKeys } from "@/lib/use-item-keys";
import { VarText } from "../var-text";
import { ColorChip, ColorPopover } from "./color-picker";
import { trimNumber, useDecimalInput } from "./decimal";
import { Help } from "./field-shell";
import { parseListNumbers, type LengthLimits } from "./list-values";
import type { FieldProps } from "./types";

type Keyed<T> = { item: T; key: string };

export function DragHandle({ label }: { label: string }) {
  return (
    <SortableItemHandle
      aria-label={`Drag to reorder ${label}`}
      className="text-muted-foreground hover:text-foreground flex h-8 w-4 shrink-0 items-center justify-center"
    >
      <GripVertical className="size-3.5" />
    </SortableItemHandle>
  );
}

export function SortableRows<T>({
  rows,
  onReorder,
  className = "flex flex-col gap-1.5",
  children,
}: {
  rows: Keyed<T>[];
  onReorder: (keys: string[]) => void;
  className?: string;
  children: (row: Keyed<T>, index: number) => ReactNode;
}) {
  return (
    <Sortable value={rows.map((r) => r.key)} onValueChange={onReorder} orientation="vertical">
      <SortableContent className={className}>
        {rows.map((row, i) => (
          <SortableItem key={row.key} value={row.key} className="flex items-start gap-1">
            {children(row, i)}
          </SortableItem>
        ))}
      </SortableContent>
    </Sortable>
  );
}

function RemoveButton({
  label,
  onClick,
  disabled,
}: {
  label: string;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <Button
      variant="ghost"
      size="icon-sm"
      className="text-muted-foreground mt-0.5 shrink-0"
      aria-label={`Remove ${label}`}
      disabled={disabled}
      onClick={onClick}
    >
      <X />
    </Button>
  );
}

function AddButton({
  onClick,
  disabled,
  children = "Add",
}: {
  onClick: () => void;
  disabled?: boolean;
  children?: ReactNode;
}) {
  return (
    <Button variant="outline" size="sm" className="w-fit" disabled={disabled} onClick={onClick}>
      <Plus data-icon="inline-start" />
      {children}
    </Button>
  );
}

function ListShell({
  label,
  meta,
  children,
}: Pick<FieldProps<unknown>, "label" | "meta"> & { children: ReactNode }) {
  return (
    <FieldSet className="gap-2">
      <FieldLegend variant="label">{label}</FieldLegend>
      {children}
      <Help meta={meta} />
    </FieldSet>
  );
}

function useFocusItem(selector = "[data-list-input]") {
  const box = useRef<HTMLDivElement>(null);
  return {
    box,
    focus: (index: number) =>
      requestAnimationFrame(() => {
        const items = box.current?.querySelectorAll<HTMLElement>(selector);
        const el = items?.[Math.max(0, index)];
        el?.focus();
      }),
  };
}

const itemName = (label: string, i: number) => `${label.toLowerCase()} ${i + 1}`;

export function StringListField({
  id,
  label,
  meta,
  value,
  onChange,
  ctx,
  limits,
}: FieldProps<string[]> & { limits: LengthLimits }) {
  const list = useItemKeys(value);
  const { box, focus } = useFocusItem("[role=textbox]");
  const full = value.length >= limits.max;
  return (
    <ListShell label={label} meta={meta}>
      <div ref={box}>
        <SortableRows rows={list.rows} onReorder={(keys) => onChange(list.reorder(keys))}>
          {(row, i) => (
            <>
              <DragHandle label={itemName(label, i)} />
              <div className="min-w-0 flex-1">
                <VarText
                  id={`${id}-${row.key}`}
                  value={row.item}
                  vars={ctx.vars}
                  onChange={(text) => onChange(list.update(i, text))}
                  lines={{
                    onEnter: () => {
                      if (full) return;
                      onChange(list.insert(i + 1, [""]));
                      focus(i + 1);
                    },
                    onRemoveEmpty: () => {
                      if (value.length <= limits.min) return;
                      onChange(list.remove(i));
                      focus(i - 1);
                    },
                    onPasteLines: (lines) => {
                      const room = Math.max(0, limits.max - value.length + 1);
                      const [first, ...rest] = lines.slice(0, room);
                      const replaced = list.update(i, row.item ? `${row.item}${first}` : first);
                      onChange([...replaced.slice(0, i + 1), ...rest, ...replaced.slice(i + 1)]);
                      focus(i + rest.length);
                    },
                  }}
                />
              </div>
              <RemoveButton
                label={itemName(label, i)}
                disabled={value.length <= limits.min}
                onClick={() => onChange(list.remove(i))}
              />
            </>
          )}
        </SortableRows>
      </div>
      <AddButton
        disabled={full}
        onClick={() => {
          onChange(list.add(""));
          focus(value.length);
        }}
      />
    </ListShell>
  );
}

function NumberCell({
  label,
  value,
  onChange,
  onPaste,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  onPaste: (numbers: number[]) => void;
}) {
  const input = useDecimalInput(trimNumber(value), "edit", (text) => {
    const n = parseNumberText(text);
    if (n !== null) onChange(n);
  });
  return (
    <Input
      data-list-input
      aria-label={label}
      className="h-8 flex-1 text-right tabular-nums"
      {...input}
      onPaste={(e) => {
        const numbers = parseListNumbers(e.clipboardData.getData("text/plain"));
        if (numbers.length < 2) return;
        e.preventDefault();
        onPaste(numbers);
      }}
    />
  );
}

export function NumberListField({
  label,
  meta,
  value,
  onChange,
  limits,
}: FieldProps<number[]> & { limits: LengthLimits }) {
  const list = useItemKeys(value);
  const { box, focus } = useFocusItem();
  const full = value.length >= limits.max;
  return (
    <ListShell label={label} meta={meta}>
      <div ref={box}>
        <SortableRows rows={list.rows} onReorder={(keys) => onChange(list.reorder(keys))}>
          {(row, i) => (
            <>
              <DragHandle label={itemName(label, i)} />
              <NumberCell
                label={itemName(label, i)}
                value={row.item}
                onChange={(n) => onChange(list.update(i, n))}
                onPaste={(numbers) => {
                  const room = Math.max(1, limits.max - value.length + 1);
                  const fitted = numbers.slice(0, room);
                  onChange([...value.slice(0, i), ...fitted, ...value.slice(i + 1)]);
                }}
              />
              <RemoveButton
                label={itemName(label, i)}
                disabled={value.length <= limits.min}
                onClick={() => onChange(list.remove(i))}
              />
            </>
          )}
        </SortableRows>
      </div>
      <AddButton
        disabled={full}
        onClick={() => {
          onChange(list.add(value.at(-1) ?? 0));
          focus(value.length);
        }}
      />
    </ListShell>
  );
}

export function ColorListField({
  label,
  meta,
  value,
  onChange,
  ctx,
  limits,
}: FieldProps<string[]> & { limits: LengthLimits }) {
  const list = useItemKeys(value);
  const full = value.length >= limits.max;
  return (
    <ListShell label={label} meta={meta}>
      <Sortable
        value={list.rows.map((r) => r.key)}
        onValueChange={(keys) => onChange(list.reorder(keys))}
        orientation="mixed"
      >
        <SortableContent className="flex flex-wrap items-center gap-1.5">
          {list.rows.map((row, i) => (
            <SortableItem key={row.key} value={row.key} asHandle className="rounded">
              <ColorPopover
                label={`Colour ${i + 1}`}
                value={row.item}
                swatches={ctx.swatches}
                onChange={(c) => onChange(list.update(i, c))}
                trigger={<ColorChip value={row.item} className="size-8 rounded-md" />}
                footer={
                  <Button
                    variant="ghost"
                    size="sm"
                    className="text-muted-foreground w-fit"
                    disabled={value.length <= limits.min}
                    onClick={() => onChange(list.remove(i))}
                  >
                    <Trash2 data-icon="inline-start" />
                    Remove colour
                  </Button>
                }
              />
            </SortableItem>
          ))}
          <Button
            variant="outline"
            size="icon"
            className="size-8 border-dashed"
            aria-label="Add colour"
            title="Add colour"
            disabled={full}
            onClick={() => onChange(list.add(value.at(-1) ?? "#ffffff"))}
          >
            <Plus />
          </Button>
        </SortableContent>
      </Sortable>
    </ListShell>
  );
}
