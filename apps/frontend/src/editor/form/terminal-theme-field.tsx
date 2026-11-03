import { Check, ChevronsUpDown } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import type { TerminalTheme } from "@/generated/bindings";
import { cn } from "@/lib/utils";
import { terminalThemes } from "@/store/catalog";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

const SHOWN_COLOURS = 8;

function Swatches({ theme }: { theme: TerminalTheme }) {
  return (
    <span
      className="flex shrink-0 items-center gap-0.5 rounded-sm border p-0.5"
      style={{ background: theme.bg }}
    >
      <span className="px-0.5 font-mono text-[10px] leading-none" style={{ color: theme.fg }}>
        Aa
      </span>
      {theme.palette.slice(0, SHOWN_COLOURS).map((colour, i) => (
        <span key={i} className="size-2 rounded-[2px]" style={{ background: colour }} />
      ))}
    </span>
  );
}

export function TerminalThemeField({ id, label, value, onChange, meta }: FieldProps<string>) {
  const [open, setOpen] = useState(false);
  const themes = terminalThemes();
  const current = themes.find((t) => t.name === value);
  const own = meta.zeroLabel ?? "";
  function pick(name: string) {
    onChange(name);
    setOpen(false);
  }
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger
          render={
            <Button id={id} variant="outline" className="w-full justify-between gap-2 font-normal">
              {current ? <Swatches theme={current} /> : null}
              <span className={cn("flex-1 truncate text-left", !value && "text-muted-foreground")}>
                {value || own}
              </span>
              <ChevronsUpDown data-icon="inline-end" />
            </Button>
          }
        />
        <PopoverContent className="w-80 p-0" align="start">
          <Command>
            <CommandInput placeholder="Search themes…" />
            <CommandList>
              <CommandEmpty>No theme found.</CommandEmpty>
              <CommandGroup>
                {own ? (
                  <CommandItem value={own} onSelect={() => pick("")}>
                    <span className="flex-1">{own}</span>
                    {!value ? <Check data-icon="inline-end" /> : null}
                  </CommandItem>
                ) : null}
                {themes.map((t) => (
                  <CommandItem key={t.name} value={t.name} onSelect={() => pick(t.name)}>
                    <Swatches theme={t} />
                    <span className="flex-1 truncate">{t.name}</span>
                    {value === t.name ? <Check data-icon="inline-end" /> : null}
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </FieldShell>
  );
}
