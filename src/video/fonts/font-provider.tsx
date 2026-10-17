import React, { useLayoutEffect, useMemo, useRef, useState } from "react";
import { continueRender, delayRender } from "remotion";
import { firstFamily } from "../../shared/fonts";
import { DEFAULT_PROJECT_FONTS, type ProjectFonts } from "../../shared/fonts/types";
import type { GroupProps } from "../../shared/compile/clip";
import type { TimelineTrack } from "../../shared/schema/timeline";
import { resolveAssetRef, useAssetBase } from "../assets";
import { applyProjectFonts } from "./registry";

export function usedFontNames(tracks: TimelineTrack[]): string[] {
  const out = new Set<string>();
  const visit = (list: TimelineTrack[]) => {
    for (const track of list)
      for (const item of track.items) {
        const font = item.props.fontFamily;
        if (typeof font === "string" && font.trim()) out.add(firstFamily(font));
        const inner = (item.props as unknown as GroupProps).tracks;
        if (Array.isArray(inner)) visit(inner);
        const wrapper = (item.props as unknown as GroupProps).wrapper?.props?.fontFamily;
        if (typeof wrapper === "string" && wrapper.trim()) out.add(firstFamily(wrapper));
      }
  };
  visit(tracks);
  return [...out].toSorted();
}

export const FontProvider: React.FC<{
  fonts?: ProjectFonts;
  tracks: TimelineTrack[];
  children: React.ReactNode;
}> = ({ fonts, tracks, children }) => {
  const base = useAssetBase();
  const value = fonts ?? DEFAULT_PROJECT_FONTS;
  const used = useMemo(() => usedFontNames(tracks), [tracks]);
  const key = JSON.stringify([value, used, base]);
  const [firstHandle] = useState(() => delayRender("Loading fonts"));
  const first = useRef(true);
  const [ready, setReady] = useState(false);

  useLayoutEffect(() => {
    const handle = first.current ? firstHandle : delayRender("Loading fonts");
    first.current = false;
    let open = true;
    const finish = () => {
      if (!open) return;
      open = false;
      continueRender(handle);
    };
    void applyProjectFonts(value, (ref) => resolveAssetRef(ref, base), used).then(() => {
      setReady(true);
      finish();
    }, finish);
    return finish;
    // oxlint-disable-next-line react/exhaustive-deps
  }, [key]);

  return ready ? <>{children}</> : null;
};
