import { appRelPath, isExternalRef, projectFileUrl } from "@/shared/asset-ref";

export function fileUrl(ref: string): string {
  const app = appRelPath(ref);
  if (app !== null) return new URL(app, document.baseURI).href;
  return isExternalRef(ref) ? ref : projectFileUrl(ref);
}
