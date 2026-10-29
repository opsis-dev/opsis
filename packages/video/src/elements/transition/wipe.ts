import { wipe as wipeOf } from "@remotion/transitions/wipe";
import type { ElementSettings } from "#video/generated/spec.ts";
import { presentation } from "#video/registry/framing.tsx";

export const wipe = presentation(({ direction }: ElementSettings["wipe"]) => wipeOf({ direction }));
