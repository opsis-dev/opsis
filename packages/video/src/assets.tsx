import { createContext, useContext } from "react";
import { staticFile } from "remotion";
import {
  PREPARED_FOLDER,
  STATIC_PROJECT_BASE,
  isAssetRef,
  preparedPath,
  preparedSource,
  joinUrl,
  refTarget,
  versionedUrl,
  type FileVersions,
} from "#video/common/asset-ref.ts";
import type { ElementProps } from "#video/common/spec.ts";
import type { PreviewQuality, ReframeMode } from "#video/generated/spec.ts";

export type AssetBase = { projectBase?: string; versions?: FileVersions };

const AssetContext = createContext<AssetBase>({});

export const AssetProvider = AssetContext.Provider;

export const useAssetBase = (): AssetBase => useContext(AssetContext);

function resolvePrepared(path: string, base: AssetBase): string {
  if (base.projectBase === undefined) return `prepared:${path}`;
  if (base.projectBase === STATIC_PROJECT_BASE)
    return staticFile(`project/${PREPARED_FOLDER}/${path}`);
  const url = joinUrl(base.projectBase, `${PREPARED_FOLDER}/${path}`);
  return versionedUrl(url, preparedSource(path) ?? "", base.versions);
}

export function resolveAssetRef(ref: string, base: AssetBase): string {
  const prepared = preparedPath(ref);
  if (prepared !== null) return resolvePrepared(prepared, base);
  const target = refTarget(ref);
  if (target.kind === "app") return staticFile(target.rel);
  if (target.kind !== "project") return ref;
  if (base.projectBase === undefined) return ref;
  if (base.projectBase === STATIC_PROJECT_BASE) return staticFile(`project/${target.rel}`);
  return versionedUrl(joinUrl(base.projectBase, target.rel), ref, base.versions);
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
