type SoundFile = {
  id: string;
  label: string;
  durationMs: number;
  credit: string;
  source: string;
};

type BuiltinSound = SoundFile & { ref: `app:sfx/${string}.wav` };

const SOUND_FILES: SoundFile[] = [
  {
    id: "whoosh",
    label: "Whoosh",
    durationMs: 150,
    credit: "Woosh by 1bob",
    source: "https://freesound.org/s/831936/",
  },
  {
    id: "whip",
    label: "Whip",
    durationMs: 170,
    credit: "SWSH_Badminton Racquet_Recording_01_JW Audio by JW_Audio",
    source: "https://freesound.org/s/838766/",
  },
  {
    id: "mouse-click",
    label: "Mouse click",
    durationMs: 400,
    credit: "Mouse Click Sound.mp3 by Pixeliota",
    source: "https://freesound.org/s/678248/",
  },
  {
    id: "switch",
    label: "Switch",
    durationMs: 330,
    credit: "UI Audio - Switch 35 by Kenney",
    source: "https://kenney.nl/",
  },
  {
    id: "page-turn",
    label: "Page turn",
    durationMs: 400,
    credit: "Draw Knife 1 by Kenney",
    source: "https://kenney.nl/",
  },
  {
    id: "shutter-modern",
    label: "Camera shutter",
    durationMs: 490,
    credit: "DSLR Shutter fast 006.wav by ristooooo1",
    source: "https://freesound.org/s/539136/",
  },
  {
    id: "shutter-old",
    label: "Old camera shutter",
    durationMs: 310,
    credit: "Werra.wav by hmilleo",
    source: "https://freesound.org/s/409093/",
  },
];

export const BUILTIN_SOUNDS: BuiltinSound[] = SOUND_FILES.map((s) => ({
  ...s,
  ref: `app:sfx/${s.id}.wav`,
}));

export const builtinSound = (ref: string): BuiltinSound | undefined =>
  BUILTIN_SOUNDS.find((s) => s.ref === ref);
