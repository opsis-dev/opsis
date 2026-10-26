import { joinUrl, libraryRelPath, refTarget } from "@opsis/video/common/asset-ref";
import { commands } from "@/generated/bindings";

let fileRoot = "";

export async function loadFileRoot(): Promise<void> {
  fileRoot = await commands.fileRoot();
}

export function projectFilesUrl(): string {
  return `${fileRoot}project/`;
}

export function fileUrl(ref: string): string {
  const target = refTarget(ref);
  if (target.kind === "project") return joinUrl(projectFilesUrl(), target.rel);
  if (target.kind === "app") return joinUrl(`${fileRoot}app/`, target.rel);
  return ref;
}

export function fontFileUrl(ref: string): string {
  const rel = libraryRelPath(ref);
  return rel === null ? fileUrl(ref) : joinUrl(`${fileRoot}fonts/`, rel);
}
