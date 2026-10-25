import { walkItems } from "@opsis/core/compile/clip";
import { firstFamily } from "@opsis/core/fonts";
import { DEFAULT_PROJECT_FONTS, type ProjectFonts } from "@opsis/core/fonts/types";
import { nonBlankString } from "@opsis/core/guards";
import type { TimelineTrack } from "@opsis/core/schema/timeline";
import React, { useMemo } from "react";
import { resolveAssetRef, useAssetBase, type AssetBase } from "../assets";
import { useHeldLoad } from "../core/loading";
import { applyProjectFonts } from "./registry";

export function usedFontNames(tracks: TimelineTrack[]): string[] {
  const out = new Set<string>();
  const add = (font: unknown) => {
    const name = nonBlankString(font);
    if (name) out.add(firstFamily(name));
  };
  walkItems(tracks, (item) => {
    if (item.kind === "element") add(item.props.fontFamily);
    if (item.kind === "group") add(item.props.wrapper?.props.fontFamily);
  });
  return [...out].toSorted();
}

function useFontsReady(fonts: ProjectFonts, used: string[], base: AssetBase): boolean {
  const loaded = useHeldLoad("Loading fonts", JSON.stringify({ fonts, used, base }), () =>
    applyProjectFonts(fonts, (ref) => resolveAssetRef(ref, base), used),
  );
  return loaded !== null;
}

export const FontProvider: React.FC<{
  fonts?: ProjectFonts;
  tracks: TimelineTrack[];
  children: React.ReactNode;
}> = ({ fonts, tracks, children }) => {
  const base = useAssetBase();
  const used = useMemo(() => usedFontNames(tracks), [tracks]);
  const ready = useFontsReady(fonts ?? DEFAULT_PROJECT_FONTS, used, base);
  return ready ? <>{children}</> : null;
};
