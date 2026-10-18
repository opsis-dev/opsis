import React, { useLayoutEffect, useMemo, useState } from "react";
import { cancelRender } from "remotion";
import { groupOf, walkItems } from "../../shared/compile/clip";
import { firstFamily } from "../../shared/fonts";
import { DEFAULT_PROJECT_FONTS, type ProjectFonts } from "../../shared/fonts/types";
import type { TimelineTrack } from "../../shared/schema/timeline";
import { resolveAssetRef, useAssetBase } from "../assets";
import { holdRender, useStableValue } from "../core/loading";
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

type FontRequest = { fonts: ProjectFonts; used: string[]; projectBase?: string };

function useFontsReady(request: FontRequest): boolean {
  const { fonts, used, projectBase } = useStableValue(request);
  const [ready, setReady] = useState(false);

  useLayoutEffect(() => {
    const release = holdRender("Loading fonts");
    void applyProjectFonts(fonts, (ref) => resolveAssetRef(ref, { projectBase }), used).then(
      () => {
        setReady(true);
        release();
      },
      (error: unknown) => cancelRender(error),
    );
    return release;
  }, [fonts, used, projectBase]);

  return ready;
}

export const FontProvider: React.FC<{
  fonts?: ProjectFonts;
  tracks: TimelineTrack[];
  children: React.ReactNode;
}> = ({ fonts, tracks, children }) => {
  const { projectBase } = useAssetBase();
  const used = useMemo(() => usedFontNames(tracks), [tracks]);
  const request = useMemo(
    () => ({ fonts: fonts ?? DEFAULT_PROJECT_FONTS, used, projectBase }),
    [fonts, used, projectBase],
  );
  const ready = useFontsReady(request);
  return ready ? <>{children}</> : null;
};
