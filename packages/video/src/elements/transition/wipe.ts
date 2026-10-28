import { wipe as wipeOf } from "@remotion/transitions/wipe";
import { CORNERS, directional, EDGES } from "#video/lib/transitions/directional.ts";

export const wipe = directional(wipeOf, [...EDGES, ...CORNERS], "from-left");
