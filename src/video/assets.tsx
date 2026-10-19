import { createContext, useContext } from "react";
import { staticFile } from "remotion";
import {
  PROJECT_FILE_BASE,
  STATIC_PROJECT_BASE,
  isAssetRef,
  joinUrl,
  refTarget,
} from "../shared/asset-ref";
import type { ElementProps } from "../shared/elements/types";
import type { PreviewQuality, ReframeMode } from "../shared/schema/timeline";

type AssetBase = { projectBase?: string };

const AssetContext = createContext<AssetBase>({});

export const AssetProvider = AssetContext.Provider;

export const useAssetBase = (): AssetBase => useContext(AssetContext);

export function resolveAssetRef(ref: string, base: AssetBase): string {
  const target = refTarget(ref);
  if (target.kind === "app") return staticFile(target.rel);
  if (target.kind !== "project") return ref;
  if (base.projectBase === STATIC_PROJECT_BASE) return staticFile(`project/${target.rel}`);
  return joinUrl(base.projectBase ?? PROJECT_FILE_BASE, target.rel);
}

const MAX_PROP_DEPTH = 3;

const isNestedTimeline = (v: object, depth: number) => depth > 0 && "tracks" in v;

export function resolveProps(props: ElementProps, base: AssetBase): ElementProps {
  const visit = (v: unknown, depth: number): unknown => {
    if (isAssetRef(v)) return resolveAssetRef(v, base);
    if (depth > MAX_PROP_DEPTH || v === null || typeof v !== "object") return v;
    if (Array.isArray(v)) return v.map((x) => visit(x, depth + 1));
    if (isNestedTimeline(v, depth)) return v;
    return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, visit(x, depth + 1)]));
  };
  return Object.fromEntries(Object.entries(props).map(([k, v]) => [k, visit(v, 1)]));
}

export const useAsset = (ref: string | null | undefined): string | null => {
  const base = useAssetBase();
  return ref ? resolveAssetRef(ref, base) : null;
};

export const DEFAULT_REFRAME: ReframeMode = "follow";
const ReframeContext = createContext<ReframeMode>(DEFAULT_REFRAME);
export const ReframeProvider = ReframeContext.Provider;
export const useReframe = (): ReframeMode => useContext(ReframeContext);

export const DEFAULT_QUALITY: PreviewQuality = "exact";
const QualityContext = createContext<PreviewQuality>(DEFAULT_QUALITY);
export const QualityProvider = QualityContext.Provider;
export const useQuality = (): PreviewQuality => useContext(QualityContext);
