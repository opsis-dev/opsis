import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { readPaths } from "./lib/setup/store";
import { ROOT, devCache } from "./lib/tool-config";

const OUT = path.join(ROOT, "public", "environments", "room");
const WIDTH = 1024;
const HEIGHT = 512;
const CUBE_SIZE = 256;
const SCENE_DROP = -3.5;

type Vec = [number, number, number];

type Box = {
  position: Vec;
  scale: Vec;
  turn: number;
  emissive?: number;
  inside?: boolean;
};

const LIGHT = { position: [0.418, 16.199, 0.3] as Vec, intensity: 900, distance: 28, decay: 2 };

const BOXES: Box[] = [
  { position: [-0.757, 13.219, 0.717], scale: [31.713, 28.305, 28.591], turn: 0, inside: true },
  { position: [-10.906, 2.009, 1.846], scale: [2.328, 7.905, 4.651], turn: -0.195 },
  { position: [-5.607, -0.754, -0.758], scale: [1.97, 1.534, 3.955], turn: 0.994 },
  { position: [6.167, 0.857, 7.803], scale: [3.927, 6.285, 3.687], turn: 0.561 },
  { position: [-2.017, 0.018, 6.124], scale: [2.002, 4.566, 2.064], turn: 0.333 },
  { position: [2.291, -0.756, -2.621], scale: [1.546, 1.552, 1.496], turn: -0.286 },
  { position: [-2.193, -0.369, -5.547], scale: [3.875, 3.487, 2.986], turn: 0.516 },
  { position: [-16.116, 14.37, 8.208], scale: [0.1, 2.428, 2.739], turn: 0, emissive: 50 },
  { position: [-16.109, 18.021, -8.207], scale: [0.1, 2.425, 2.751], turn: 0, emissive: 50 },
  { position: [14.904, 12.198, -1.832], scale: [0.15, 4.265, 6.331], turn: 0, emissive: 17 },
  { position: [-0.462, 8.89, 14.52], scale: [4.38, 5.441, 0.088], turn: 0, emissive: 43 },
  { position: [3.235, 11.486, -12.541], scale: [2.5, 2.0, 0.1], turn: 0, emissive: 20 },
  { position: [0.0, 20.0, 0.0], scale: [1.0, 0.1, 1.0], turn: 0, emissive: 100 },
];

function sub(a: Vec, b: Vec): Vec {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function dot(a: Vec, b: Vec): number {
  return a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
}

function scaled(a: Vec, s: number): Vec {
  return [a[0] * s, a[1] * s, a[2] * s];
}

function normalized(a: Vec): Vec {
  return scaled(a, 1 / Math.hypot(a[0], a[1], a[2]));
}

function turnedY(v: Vec, angle: number): Vec {
  const c = Math.cos(angle);
  const s = Math.sin(angle);
  return [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c];
}

type Hit = { distance: number; normal: Vec; box: Box };

function hitBox(origin: Vec, direction: Vec, box: Box): Hit | null {
  const placed: Vec = [box.position[0], box.position[1] + SCENE_DROP, box.position[2]];
  const local = turnedY(sub(origin, placed), -box.turn);
  const dir = turnedY(direction, -box.turn);
  const half = scaled(box.scale, 0.5);
  let near = -Infinity;
  let far = Infinity;
  let nearAxis = 0;
  let farAxis = 0;
  for (let axis = 0; axis < 3; axis++) {
    const o = local[axis] ?? 0;
    const d = dir[axis] ?? 0;
    const h = half[axis] ?? 0;
    if (Math.abs(d) < 1e-12) {
      if (o < -h || o > h) return null;
      continue;
    }
    let t0 = (-h - o) / d;
    let t1 = (h - o) / d;
    if (t0 > t1) [t0, t1] = [t1, t0];
    if (t0 > near) {
      near = t0;
      nearAxis = axis;
    }
    if (t1 < far) {
      far = t1;
      farAxis = axis;
    }
  }
  if (near > far) return null;
  const inside = box.inside === true;
  const distance = inside ? far : near;
  if (distance <= 1e-6) return null;
  const axis = inside ? farAxis : nearAxis;
  const at = (local[axis] ?? 0) + (dir[axis] ?? 0) * distance;
  const outward: Vec = [0, 0, 0];
  outward[axis] = Math.sign(at);
  const normal = turnedY(inside ? scaled(outward, -1) : outward, box.turn);
  return { distance, normal, box };
}

function attenuation(distance: number): number {
  const falloff = 1 / Math.max(distance ** LIGHT.decay, 0.01);
  const window = Math.max(0, Math.min(1, 1 - (distance / LIGHT.distance) ** 4));
  return falloff * window * window;
}

function shade(direction: Vec): number {
  let nearest: Hit | null = null;
  for (const box of BOXES) {
    const hit = hitBox([0, 0, 0], direction, box);
    if (hit && (!nearest || hit.distance < nearest.distance)) nearest = hit;
  }
  if (!nearest) return 0;
  if (nearest.box.emissive !== undefined) return nearest.box.emissive;
  const point = scaled(direction, nearest.distance);
  const light: Vec = [LIGHT.position[0], LIGHT.position[1] + SCENE_DROP, LIGHT.position[2]];
  const toLight = sub(light, point);
  const distance = Math.hypot(toLight[0], toLight[1], toLight[2]);
  const ndotl = Math.max(0, dot(nearest.normal, normalized(toLight)));
  return (LIGHT.intensity * attenuation(distance) * ndotl) / Math.PI;
}

function rgbe(value: number): [number, number, number, number] {
  if (value < 1e-32) return [0, 0, 0, 0];
  const exponent = Math.ceil(Math.log2(value));
  const mantissa = Math.min(255, Math.floor((value / 2 ** exponent) * 256));
  return [mantissa, mantissa, mantissa, exponent + 128];
}

function radiance(): Buffer {
  const header = Buffer.from(`#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y ${HEIGHT} +X ${WIDTH}\n`);
  const pixels = Buffer.alloc(WIDTH * HEIGHT * 4);
  for (let y = 0; y < HEIGHT; y++) {
    const theta = ((y + 0.5) / HEIGHT) * Math.PI;
    for (let x = 0; x < WIDTH; x++) {
      const phi = ((x + 0.5) / WIDTH) * Math.PI * 2;
      const direction: Vec = [
        Math.sin(theta) * Math.sin(phi),
        Math.cos(theta),
        -Math.sin(theta) * Math.cos(phi),
      ];
      pixels.set(rgbe(shade(direction)), (y * WIDTH + x) * 4);
    }
  }
  return Buffer.concat([header, pixels]);
}

const SH_LINE = /\(\s*(-?[\d.]+),\s*(-?[\d.]+),\s*(-?[\d.]+)\)/;

function shCoefficients(file: string): number[][] {
  return readFileSync(file, "utf8")
    .split("\n")
    .map((line) => SH_LINE.exec(line))
    .filter((found) => found !== null)
    .map((found) => [Number(found[1]), Number(found[2]), Number(found[3])]);
}

function main(): void {
  const work = devCache("room-environment");
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
  const source = path.join(work, "room.hdr");
  writeFileSync(source, radiance());
  const cmgen = path.join(readPaths().filament, "bin", "cmgen");
  const deployed = path.join(work, "deploy");
  execFileSync(
    cmgen,
    ["--quiet", `--size=${CUBE_SIZE}`, "--format=ktx", `--deploy=${deployed}`, source],
    { stdio: "inherit" },
  );
  rmSync(OUT, { recursive: true, force: true });
  mkdirSync(OUT, { recursive: true });
  copyFileSync(path.join(deployed, "deploy_ibl.ktx"), path.join(OUT, "ibl.ktx"));
  writeFileSync(
    path.join(OUT, "sh.json"),
    `${JSON.stringify(shCoefficients(path.join(deployed, "sh.txt")))}\n`,
  );
  console.log(
    `room environment written to ${path.relative(ROOT, OUT)}: ${readdirSync(OUT).join(", ")}`,
  );
}

main();
