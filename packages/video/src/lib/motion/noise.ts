import { random } from "@opsis/runtime/random.ts";
import { createNoise3D, type NoiseFunction3D } from "simplex-noise";

const fields = new Map<string, NoiseFunction3D>();

export function noiseField(seed: string | number): NoiseFunction3D {
  const key = String(seed);
  const known = fields.get(key);
  if (known) return known;
  let draws = 0;
  function next(): number {
    draws += 1;
    return random(`${key}:${draws}`);
  }
  const field = createNoise3D(next);
  fields.set(key, field);
  return field;
}
