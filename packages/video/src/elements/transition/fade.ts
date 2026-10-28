import { fade as fadeOf } from "@remotion/transitions/fade";
import { presentation } from "#video/registry/framing.tsx";

export const fade = presentation(() => fadeOf());
