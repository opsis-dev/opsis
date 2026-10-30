import { commands } from "@/generated/bindings";

let fileRoot = "";

export async function loadFileRoot(): Promise<void> {
  fileRoot = await commands.fileRoot();
}

export function fileUrl(ref: string): string {
  return `${fileRoot}${encodeURIComponent(ref)}`;
}
