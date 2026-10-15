export type BuiltinSound = {
  id: string;
  label: string;
  ref: `app:sfx/${string}.wav`;
  durationMs: number;
  credit: string;
  source: string;
};

export const BUILTIN_SOUNDS: BuiltinSound[] = [
  {
    id: "whoosh",
    label: "Whoosh",
    ref: "app:sfx/whoosh.wav",
    durationMs: 150,
    credit: "Woosh by 1bob",
    source: "https://freesound.org/s/831936/",
  },
  {
    id: "whip",
    label: "Whip",
    ref: "app:sfx/whip.wav",
    durationMs: 170,
    credit: "SWSH_Badminton Racquet_Recording_01_JW Audio by JW_Audio",
    source: "https://freesound.org/s/838766/",
  },
  {
    id: "mouse-click",
    label: "Mouse click",
    ref: "app:sfx/mouse-click.wav",
    durationMs: 400,
    credit: "Mouse Click Sound.mp3 by Pixeliota",
    source: "https://freesound.org/s/678248/",
  },
  {
    id: "switch",
    label: "Switch",
    ref: "app:sfx/switch.wav",
    durationMs: 330,
    credit: "UI Audio - Switch 35 by Kenney",
    source: "https://kenney.nl/",
  },
  {
    id: "page-turn",
    label: "Page turn",
    ref: "app:sfx/page-turn.wav",
    durationMs: 400,
    credit: "Draw Knife 1 by Kenney",
    source: "https://kenney.nl/",
  },
  {
    id: "shutter-modern",
    label: "Camera shutter",
    ref: "app:sfx/shutter-modern.wav",
    durationMs: 490,
    credit: "DSLR Shutter fast 006.wav by ristooooo1",
    source: "https://freesound.org/s/539136/",
  },
  {
    id: "shutter-old",
    label: "Old camera shutter",
    ref: "app:sfx/shutter-old.wav",
    durationMs: 310,
    credit: "Werra.wav by hmilleo",
    source: "https://freesound.org/s/409093/",
  },
];

export const builtinSound = (ref: string): BuiltinSound | undefined =>
  BUILTIN_SOUNDS.find((s) => s.ref === ref);
