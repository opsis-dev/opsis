type SoundFile = {
  id: string;
  label: string;
  durationMs: number;
};

type BuiltinSound = SoundFile & { ref: `app:sfx/${string}.wav` };

const SOUND_FILES: SoundFile[] = [
  {
    id: "whoosh",
    label: "Whoosh",
    durationMs: 150,
  },
  {
    id: "whip",
    label: "Whip",
    durationMs: 170,
  },
  {
    id: "mouse-click",
    label: "Mouse click",
    durationMs: 400,
  },
  {
    id: "switch",
    label: "Switch",
    durationMs: 330,
  },
  {
    id: "page-turn",
    label: "Page turn",
    durationMs: 400,
  },
  {
    id: "shutter-modern",
    label: "Camera shutter",
    durationMs: 490,
  },
  {
    id: "shutter-old",
    label: "Old camera shutter",
    durationMs: 310,
  },
];

export const BUILTIN_SOUNDS: BuiltinSound[] = SOUND_FILES.map((s) => ({
  ...s,
  ref: `app:sfx/${s.id}.wav`,
}));

export const builtinSound = (ref: string): BuiltinSound | undefined =>
  BUILTIN_SOUNDS.find((s) => s.ref === ref);
