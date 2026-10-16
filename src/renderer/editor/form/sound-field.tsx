import { Play } from "lucide-react";
import { toast } from "sonner";
import { baseName } from "@/shared/asset-ref";
import { BUILTIN_SOUNDS, builtinSound } from "@/shared/sounds";
import { Button } from "@/renderer/components/ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/renderer/components/ui/select";
import { fileUrl } from "../file-url";
import { FieldShell } from "./field-shell";
import type { FieldProps } from "./types";

const OWN_FILE = "own-file";

const play = (ref: string) =>
  void new Audio(fileUrl(ref)).play().catch(() => toast.error("Couldn't play this sound."));

export function SoundField({ id, name, label, value, onChange, meta, ctx }: FieldProps<string>) {
  const builtin = builtinSound(value);
  const ownLabel = value && !builtin ? baseName(value) : "Your own file…";
  const items = [
    ...BUILTIN_SOUNDS.map((s) => ({ value: s.ref as string, label: s.label })),
    { value: OWN_FILE, label: ownLabel },
  ];
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <div className="flex gap-2">
        <Select
          items={items}
          value={builtin ? builtin.ref : value ? OWN_FILE : null}
          onValueChange={(v) => {
            if (v === OWN_FILE) ctx.chooseAsset?.(name);
            else if (v != null) onChange(String(v));
          }}
        >
          <SelectTrigger id={id} className="min-w-0 flex-1">
            <SelectValue placeholder="Choose a sound" />
          </SelectTrigger>
          <SelectContent>
            <SelectGroup>
              {BUILTIN_SOUNDS.map((s) => (
                <SelectItem key={s.id} value={s.ref}>
                  {s.label}
                </SelectItem>
              ))}
            </SelectGroup>
            <SelectGroup>
              <SelectItem value={OWN_FILE} disabled={!ctx.projectDir}>
                {ownLabel}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
        <Button
          variant="outline"
          size="icon"
          aria-label="Play sound"
          disabled={!value}
          onClick={() => play(value)}
        >
          <Play />
        </Button>
      </div>
    </FieldShell>
  );
}
