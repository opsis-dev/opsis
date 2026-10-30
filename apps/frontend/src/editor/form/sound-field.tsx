import { Play, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import type { SoundChoice } from "@/generated/bindings";
import { fileName } from "@/lib/file-name";
import { builtinMusic, builtinSounds } from "@/store/catalog";
import { fileUrl } from "../file-url";
import { ChoiceSelect } from "./choice-select";
import { FieldShell } from "./field-shell";
import type { ParamFieldProps } from "./types";

const OWN_FILE = "own-file";

function lengthMsOf(ref: string): Promise<number | undefined> {
  return new Promise((resolve) => {
    const audio = new Audio();
    audio.preload = "metadata";
    audio.addEventListener("loadedmetadata", () =>
      resolve(Number.isFinite(audio.duration) ? Math.round(audio.duration * 1000) : undefined),
    );
    audio.addEventListener("error", () => resolve(undefined));
    audio.src = fileUrl(ref);
  });
}

function usePlayer(): { playing: boolean; toggle: (ref: string) => void } {
  const audio = useRef<HTMLAudioElement | null>(null);
  const [playing, setPlaying] = useState(false);
  function stop() {
    audio.current?.pause();
    audio.current = null;
    setPlaying(false);
  }
  useEffect(() => () => audio.current?.pause(), []);
  function toggle(ref: string) {
    if (audio.current) return stop();
    const player = new Audio(fileUrl(ref));
    player.addEventListener("ended", stop);
    audio.current = player;
    setPlaying(true);
    void player.play().catch(() => {
      stop();
      toast.error("Couldn't play this sound.");
    });
  }
  return { playing, toggle };
}

type Builtins = { choices: () => readonly SoundChoice[]; placeholder: string; play: string };

function BuiltinAudioField({
  id,
  label,
  value,
  onChange,
  onPatch,
  name,
  meta,
  ctx,
  builtins,
}: ParamFieldProps<string> & { builtins: Builtins }) {
  const { lengthKey } = meta;
  const { playing, toggle } = usePlayer();
  function choose(ref: string, lengthMs?: number) {
    return lengthKey ? onPatch({ [name]: ref, [lengthKey]: lengthMs }) : onChange(ref);
  }
  const choices = builtins.choices();
  const builtin = choices.find((s) => s.reference === value);
  const ownLabel = value && !builtin ? fileName(value) : "Your own file…";
  const selectedKey = builtin?.reference ?? (value ? OWN_FILE : null);
  return (
    <FieldShell id={id} label={label} meta={meta}>
      <div className="flex gap-2">
        <ChoiceSelect
          id={id}
          className="min-w-0 flex-1"
          groups={[
            choices.map((s) => ({ value: s.reference, label: s.label, hint: s.hint })),
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
          placeholder={builtins.placeholder}
          noneLabel={meta.zeroLabel}
        />
        <Button
          variant="outline"
          size="icon"
          aria-label={playing ? "Stop" : builtins.play}
          disabled={!value}
          onClick={() => toggle(value)}
        >
          {playing ? <Square /> : <Play />}
        </Button>
      </div>
    </FieldShell>
  );
}

const SOUNDS: Builtins = {
  choices: builtinSounds,
  placeholder: "Choose a sound",
  play: "Play sound",
};

const MUSIC: Builtins = { choices: builtinMusic, placeholder: "Choose a song", play: "Play song" };

export function SoundField(props: ParamFieldProps<string>) {
  return <BuiltinAudioField {...props} builtins={SOUNDS} />;
}

export function MusicField(props: ParamFieldProps<string>) {
  return <BuiltinAudioField {...props} builtins={MUSIC} />;
}
