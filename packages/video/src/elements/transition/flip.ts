import { flip as flipOf } from "@remotion/transitions/flip";
import { directional, EDGES } from "#video/lib/transitions/directional.ts";

export const flip = directional(flipOf, EDGES, "from-right");
