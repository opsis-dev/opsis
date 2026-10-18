import React, { useMemo } from "react";
import { groupOf, walkItems } from "../../shared/compile/clip";
import { firstFamily } from "../../shared/fonts";
import { DEFAULT_PROJECT_FONTS, type ProjectFonts } from "../../shared/fonts/types";
import type { TimelineTrack } from "../../shared/schema/timeline";
import { resolveAssetRef, useAssetBase } from "../assets";
import { useHeldLoad } from "../core/loading";
import { applyProjectFonts } from "./registry";

export function usedFontNames(tracks: TimelineTrack[]): string[] {
  const out = new Set<string>();
  const add = (font: unknown) => {
    if (typeof font === "string" && font.trim()) out.add(firstFamily(font));
  };
  walkItems(tracks, (item) => {
    add(item.props.fontFamily);
    add(groupOf(item)?.wrapper?.props.fontFamily);
  });
  return [...out].toSorted();
}

function useFontsReady(fonts: ProjectFonts, used: string[], projectBase?: string): boolean {
  const loaded = useHeldLoad("Loading fonts", JSON.stringify({ fonts, used, projectBase }), () =>
    applyProjectFonts(fonts, (ref) => resolveAssetRef(ref, { projectBase }), used),
  );
  return loaded !== null;
}

export const FontProvider: React.FC<{
  fonts?: ProjectFonts;
  tracks: TimelineTrack[];
  children: React.ReactNode;
}> = ({ fonts, tracks, children }) => {
  const { projectBase } = useAssetBase();
  const used = useMemo(() => usedFontNames(tracks), [tracks]);
  const ready = useFontsReady(fonts ?? DEFAULT_PROJECT_FONTS, used, projectBase);
  return ready ? <>{children}</> : null;
};
