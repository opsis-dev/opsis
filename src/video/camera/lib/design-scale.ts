import { createContext, useContext } from "react";
import { useVideoConfig } from "remotion";
import { DESIGN_WIDTH } from "./click-log";

export const DesignWidthContext = createContext<number | null>(null);

export const useDesignScale = (): number => {
  const override = useContext(DesignWidthContext);
  const { width } = useVideoConfig();
  return (override ?? width) / DESIGN_WIDTH;
};
