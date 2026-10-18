import { useState } from "react";
import { Kbd } from "@/renderer/components/ui/kbd";
import { cn } from "@/renderer/lib/utils";
import { chordFromEvent, chordKeys } from "@/shared/chord";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

export function KeysField({ id, label, value, onChange, meta }: FieldProps<string>) {
  const [listening, setListening] = useState(false);
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <button
        id={id}
        type="button"
        className={cn(
          "border-input flex min-h-8 w-full items-center gap-1 rounded-lg border px-2.5 text-left text-sm outline-none",
          "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-3",
        )}
        onFocus={() => setListening(true)}
        onBlur={() => setListening(false)}
        onKeyDown={(e) => {
          if (e.key === "Tab") return;
          e.preventDefault();
          e.stopPropagation();
          const chord = chordFromEvent(e.nativeEvent);
          if (chord) onChange(chord);
        }}
      >
        {value ? (
          chordKeys(value).map((k) => <Kbd key={k}>{k}</Kbd>)
        ) : (
          <span className="text-muted-foreground">Press a key</span>
        )}
        <span className="text-muted-foreground ml-auto text-xs">
          {listening ? "Listening…" : ""}
        </span>
      </button>
    </FieldShell>
  );
}
