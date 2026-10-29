import { commands } from "@/generated/bindings";

let fileRoot = "";

export async function loadFileRoot(): Promise<void> {
  fileRoot = await commands.fileRoot();
}

export const fileUrl = (ref: string): string => `${fileRoot}${encodeURIComponent(ref)}`;
