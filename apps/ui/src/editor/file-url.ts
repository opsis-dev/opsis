import {
  FILE_SCHEME,
  joinUrl,
  libraryRelPath,
  PROJECT_FILE_BASE,
  refTarget,
} from "@opsis/core/asset-ref";

export function fileUrl(ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return joinUrl(PROJECT_FILE_BASE, target.rel);
  if (target.kind === "app") return new URL(target.rel, document.baseURI).href;
  return ref;
}

export function fontFileUrl(ref: string): string {
  const rel = libraryRelPath(ref);
  return rel === null ? fileUrl(ref) : joinUrl(`${FILE_SCHEME}://fonts/`, rel);
}
