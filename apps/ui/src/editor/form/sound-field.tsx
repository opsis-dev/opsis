import { baseName } from "@opsis/video/common/asset-ref";
import { Play } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { BUILTIN_SOUNDS, builtinSound } from "@/core/sounds";
import { fileUrl } from "../file-url";
import { ChoiceSelect } from "./choice-select";
import { FieldShell } from "./field-shell";
import type { ParamFieldProps } from "./types";

const OWN_FILE = "own-file";

const play = (ref: string) =>
  void new Audio(fileUrl(ref)).play().catch(() => toast.error("Couldn't play this sound."));

const lengthMsOf = (ref: string): Promise<number | undefined> =>
  new Promise((resolve) => {
    const audio = new Audio();
    audio.preload = "metadata";
    audio.addEventListener("loadedmetadata", () =>
      resolve(Number.isFinite(audio.duration) ? Math.round(audio.duration * 1000) : undefined),
    );
    audio.addEventListener("error", () => resolve(undefined));
    audio.src = fileUrl(ref);
  });

export function SoundField({
  id,
  label,
  value,
  onChange,
  onPatch,
  name,
  meta,
  ctx,
}: ParamFieldProps<string>) {
  const { lengthKey } = meta;
  const choose = (ref: string, lengthMs?: number) =>
    lengthKey ? onPatch({ [name]: ref, [lengthKey]: lengthMs }) : onChange(ref);
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
            if (v === OWN_FILE)
              ctx.chooseAsset((path) => {
                if (!lengthKey) return choose(path);
                void lengthMsOf(path).then((ms) => choose(path, ms));
              });
            else choose(v ?? "");
          }}
          placeholder="Choose a sound"
          noneLabel={meta.zeroLabel}
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
