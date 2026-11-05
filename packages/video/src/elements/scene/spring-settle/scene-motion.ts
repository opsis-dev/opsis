import { Easing } from "@opsis/runtime/easing.ts";
import { random } from "@opsis/runtime/random.ts";
export const FADE = Easing.out(Easing.quad);

export function stagger(
  index: number,
  base = 5,
  power = 0.8,
  jitter = 1.5,
  seed = "stagger",
): number {
  return Math.max(0, index ** power * base + (random(`${seed}-${index}`) - 0.5) * 2 * jitter);
}

