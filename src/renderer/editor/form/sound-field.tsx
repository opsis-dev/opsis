import { Play } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/renderer/components/ui/button";
import { baseName } from "@/shared/asset-ref";
import { BUILTIN_SOUNDS, builtinSound } from "@/shared/sounds";
import { fileUrl } from "../file-url";
import { ChoiceSelect } from "./choice-select";
import { FieldShell } from "./field-shell";
import type { ParamFieldProps } from "./types";

const OWN_FILE = "own-file";

const play = (ref: string) =>
  void new Audio(fileUrl(ref)).play().catch(() => toast.error("Couldn't play this sound."));

export function SoundField({
  id,
  name,
  label,
  value,
  onChange,
  meta,
  ctx,
}: ParamFieldProps<string>) {
  const builtin = builtinSound(value);
  const ownLabel = value && !builtin ? baseName(value) : "Your own file…";
  const selectedKey = builtin?.ref ?? (value ? OWN_FILE : null);
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <div className="flex gap-2">
        <ChoiceSelect
          id={id}
          className="min-w-0 flex-1"
          groups={[
            BUILTIN_SOUNDS.map((s) => ({ value: s.ref, label: s.label })),
            [{ value: OWN_FILE, label: ownLabel, disabled: !ctx.projectDir }],
          ]}
          value={selectedKey}
          onChange={(v) => {
            if (v === OWN_FILE) ctx.chooseAsset(name);
            else onChange(v);
          }}
          placeholder="Choose a sound"
        />
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
