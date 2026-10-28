import { slide as slideOf } from "@remotion/transitions/slide";
import { directional, EDGES } from "#video/lib/transitions/directional.ts";

export const slide = directional(slideOf, EDGES, "from-right");
