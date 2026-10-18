import { encodePath, FILE_SCHEME, PROJECT_FILE_BASE, refTarget } from "@/shared/asset-ref";
import { libraryRelPath } from "@/shared/fonts/types";

export function fileUrl(ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return `${PROJECT_FILE_BASE}${encodePath(target.rel)}`;
  if (target.kind === "app") return new URL(target.rel, document.baseURI).href;
  return ref;
}

export function fontFileUrl(ref: string): string {
  const rel = libraryRelPath(ref);
  return rel === null ? fileUrl(ref) : `${FILE_SCHEME}://fonts/${encodePath(rel)}`;
}
