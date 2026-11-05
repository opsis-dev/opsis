import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { ROOT } from "../tool-config";

const INPUTS = {
  clip: "assets/clip.mp4",
  picture: "assets/picture.png",
  overlay: "assets/overlay.png",
  look: "assets/look.cube",
  cast: "assets/demo.cast",
  tape: "assets/demo.tape",
  upload: "assets/upload.txt",
  lottie: "assets/spinner.json",
  model: "assets/model.glb",
} as const;

export type InputName = keyof typeof INPUTS;

export function projectRef(name: InputName): string {
  return `project:${INPUTS[name]}`;
}

const CLIP_SECONDS = 12;

const SPEECH_LIKE = `0.4*sin(2*PI*180*t)*sin(2*PI*3*t)*between(mod(t\\,4)\\,0\\,2.6)`;

const LOTTIE = {
  v: "5.7.4",
  fr: 30,
  ip: 0,
  op: 60,
  w: 512,
  h: 512,
  nm: "spinner",
  ddd: 0,
  assets: [],
  layers: [
    {
      ddd: 0,
      ind: 1,
      ty: 4,
      nm: "box",
      sr: 1,
      ks: {
        o: { a: 0, k: 100 },
        r: {
          a: 1,
          k: [
            { t: 0, s: [0], i: { x: [0.5], y: [1] }, o: { x: [0.5], y: [0] } },
            { t: 60, s: [360] },
          ],
        },
        p: { a: 0, k: [256, 256, 0] },
        a: { a: 0, k: [0, 0, 0] },
        s: { a: 0, k: [100, 100, 100] },
      },
      ao: 0,
      shapes: [
        {
          ty: "rc",
          nm: "rect",
          d: 1,
          s: { a: 0, k: [200, 200] },
          p: { a: 0, k: [0, 0] },
          r: { a: 0, k: 24 },
        },
        { ty: "fl", nm: "fill", c: { a: 0, k: [0.23, 0.51, 0.96, 1] }, o: { a: 0, k: 100 }, r: 1 },
      ],
      ip: 0,
      op: 60,
      st: 0,
      bm: 0,
    },
  ],
};

function identityCube(): string {
  const rows = ["LUT_3D_SIZE 2"];
  for (const b of [0, 1])
    for (const g of [0, 1]) for (const r of [0, 1]) rows.push(`${r} ${g} ${b}`);
  return `${rows.join("\n")}\n`;
}

function ffmpeg(program: string, args: string[]): void {
  execFileSync(program, ["-hide_banner", "-loglevel", "error", "-y", ...args], {
    stdio: "inherit",
  });
}

export function writeInputs(project: string, ffmpegProgram: string): void {
  mkdirSync(path.join(project, "assets"), { recursive: true });
  function file(name: InputName): string {
    return path.join(project, INPUTS[name]);
  }
  ffmpeg(ffmpegProgram, [
    "-f",
    "lavfi",
    "-i",
    `testsrc2=size=1920x1080:rate=30:duration=${CLIP_SECONDS}`,
    "-f",
    "lavfi",
    "-i",
    `aevalsrc=${SPEECH_LIKE}:s=48000:d=${CLIP_SECONDS}`,
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-c:a",
    "aac",
    "-shortest",
    file("clip"),
  ]);
  ffmpeg(ffmpegProgram, [
    "-f",
    "lavfi",
    "-i",
    "testsrc2=size=1920x1080",
    "-frames:v",
    "1",
    file("picture"),
  ]);
  ffmpeg(ffmpegProgram, [
    "-f",
    "lavfi",
    "-i",
    "color=c=0x3b82f6@0.8:s=400x400,format=rgba",
    "-frames:v",
    "1",
    file("overlay"),
  ]);
  writeFileSync(file("look"), identityCube());
  writeFileSync(file("upload"), "A file the browser attaches.\n");
  writeFileSync(file("lottie"), JSON.stringify(LOTTIE));
  copyFileSync(path.join(ROOT, "crates", "terminal", "tests", "casts", "demo.cast"), file("cast"));
  copyFileSync(path.join(ROOT, "crates", "terminal", "tests", "tapes", "demo.tape"), file("tape"));
  copyFileSync(path.join(ROOT, "fixtures", "prepare", "models", "draco.glb"), file("model"));
}
