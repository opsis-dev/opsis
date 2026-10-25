import { createContext, useContext } from "react";
import { useVideoConfig } from "remotion";
import { designScaleFor } from "#video/camera/lib/design-width.ts";

export const DesignWidthContext = createContext<number | null>(null);

export const useDesignScale = (): number => {
  const override = useContext(DesignWidthContext);
  const { width } = useVideoConfig();
  return designScaleFor(override ?? width);
};
