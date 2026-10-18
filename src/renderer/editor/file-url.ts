import { encodePath, PROJECT_FILE_BASE, refTarget } from "@/shared/asset-ref";

export function fileUrl(ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return `${PROJECT_FILE_BASE}${encodePath(target.rel)}`;
  if (target.kind === "app") return new URL(target.rel, document.baseURI).href;
  return ref;
}
