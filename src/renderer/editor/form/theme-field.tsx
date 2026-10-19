import { ChevronRight, RotateCcw } from "lucide-react";
import { Button } from "@/renderer/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/renderer/components/ui/collapsible";
import { Field, FieldLabel } from "@/renderer/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/renderer/components/ui/toggle-group";
import { THEME_MODES } from "@/shared/elements/introspect";
import type { ThemeValue } from "@/shared/elements/theme";
import { humanize } from "@/shared/format";
import { isOneOf } from "@/shared/guards";
import { ColorInput } from "./color-field";
import { Help } from "./field-shell";
import type { FieldProps } from "./types";

const MODE_LABELS = { light: "Light", dark: "Dark" } as const;

export function ThemeField({ id, label, meta, value, onChange, ctx }: FieldProps<ThemeValue>) {
  const preset = meta.themes?.[value.mode] ?? {};
  const tokens = Object.keys(preset);
  const changed = Object.values(value.colors).filter((c) => c.trim()).length;
  const swatches = [...new Set([...ctx.swatches, ...Object.values(preset)])];
  const setColor = (token: string, color: string) => {
    const rest = Object.fromEntries(Object.entries(value.colors).filter(([k]) => k !== token));
    onChange({ ...value, colors: color.trim() ? { ...rest, [token]: color } : rest });
  };
  return (
    <Field>
      <div className="flex items-center justify-between gap-2">
        <FieldLabel htmlFor={id}>{label}</FieldLabel>
        <ToggleGroup
          id={id}
          variant="outline"
          size="sm"
          value={[value.mode]}
          onValueChange={(v) => {
            const mode = v[0];
            if (isOneOf(THEME_MODES, mode)) onChange({ ...value, mode });
          }}
        >
          {THEME_MODES.map((m) => (
            <ToggleGroupItem key={m} value={m}>
              {MODE_LABELS[m]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>
      {tokens.length ? (
        <Collapsible>
          <div className="flex items-center justify-between">
            <CollapsibleTrigger
              render={
                <Button
                  variant="ghost"
                  size="sm"
                  className="group/theme text-muted-foreground -ml-2 w-fit"
                />
              }
            >
              <ChevronRight
                data-icon="inline-start"
                className="transition-transform group-data-[panel-open]/theme:rotate-90"
              />
              {changed ? `Colours (${changed} changed)` : "Colours"}
            </CollapsibleTrigger>
            {changed ? (
              <Button
                variant="ghost"
                size="sm"
                className="text-muted-foreground"
                onClick={() => onChange({ ...value, colors: {} })}
              >
                <RotateCcw data-icon="inline-start" />
                Reset
              </Button>
            ) : null}
          </div>
          <CollapsibleContent className="flex flex-col gap-2 pt-2">
            {tokens.map((token) => (
              <div key={token} className="grid grid-cols-[7rem_1fr] items-center gap-2">
                <span className="text-muted-foreground truncate text-xs">{humanize(token)}</span>
                <ColorInput
                  label={humanize(token)}
                  value={value.colors[token] ?? ""}
                  emptyLabel="Theme"
                  fallback={preset[token]}
                  swatches={swatches}
                  onChange={(color) => setColor(token, color)}
                />
              </div>
            ))}
          </CollapsibleContent>
        </Collapsible>
      ) : null}
      <Help meta={meta} />
    </Field>
  );
}
