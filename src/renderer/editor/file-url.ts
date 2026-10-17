import { isExternalRef, projectFileUrl } from "@/shared/asset-ref";

export function fileUrl(ref: string): string {
  if (ref.startsWith("app:")) return new URL(ref.slice("app:".length), document.baseURI).href;
  return isExternalRef(ref) ? ref : projectFileUrl(ref);
}
