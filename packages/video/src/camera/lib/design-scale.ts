import { useFilm } from "@opsis/runtime/clock.tsx";
import { createContext, useContext } from "react";
import { designScaleFor } from "#video/camera/lib/design-width.ts";

export const DesignWidthContext = createContext<number | null>(null);

export function useDesignScale(): number {
  const override = useContext(DesignWidthContext);
  const { width } = useFilm();
  return designScaleFor(override ?? width);
}
