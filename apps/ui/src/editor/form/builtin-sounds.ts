import SOUND_FILES from "../../../../../crates/compile/src/data/sounds.json";

type BuiltinSound = { id: string; label: string; durationMs: number; ref: string };

export const BUILTIN_SOUNDS: BuiltinSound[] = SOUND_FILES.map((s) => ({
  ...s,
  ref: `app:sfx/${s.id}.wav`,
}));

export const builtinSound = (ref: string): BuiltinSound | undefined =>
  BUILTIN_SOUNDS.find((s) => s.ref === ref);
