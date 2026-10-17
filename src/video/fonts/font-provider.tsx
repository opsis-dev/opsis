import React, { useLayoutEffect, useMemo, useRef, useState } from "react";
import { continueRender, delayRender } from "remotion";
import { firstFamily } from "../../shared/fonts";
import { DEFAULT_PROJECT_FONTS, type ProjectFonts } from "../../shared/fonts/types";
import { groupOf, walkItems } from "../../shared/compile/clip";
import type { TimelineTrack } from "../../shared/schema/timeline";
import { resolveAssetRef, useAssetBase } from "../assets";
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
  const key = JSON.stringify(request);
  const [firstHandle] = useState(() => delayRender("Loading fonts"));
  const first = useRef(true);
  const [ready, setReady] = useState(false);

  useLayoutEffect(() => {
    const { fonts, used, projectBase }: FontRequest = JSON.parse(key);
    const handle = first.current ? firstHandle : delayRender("Loading fonts");
    first.current = false;
    let open = true;
    const finish = () => {
      if (!open) return;
      open = false;
      continueRender(handle);
    };
    void applyProjectFonts(fonts, (ref) => resolveAssetRef(ref, { projectBase }), used).then(() => {
      setReady(true);
      finish();
    }, finish);
    return finish;
  }, [key, firstHandle]);

  return ready;
}

export const FontProvider: React.FC<{
  fonts?: ProjectFonts;
  tracks: TimelineTrack[];
  children: React.ReactNode;
}> = ({ fonts, tracks, children }) => {
  const { projectBase } = useAssetBase();
  const used = useMemo(() => usedFontNames(tracks), [tracks]);
  const ready = useFontsReady({ fonts: fonts ?? DEFAULT_PROJECT_FONTS, used, projectBase });
  return ready ? <>{children}</> : null;
};
