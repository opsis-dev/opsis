import { safeName } from "./safe-name";

export const projectFolderName = (name: string): string => safeName(name, { fallback: "demo" });

export function joinFolder(parent: string, child: string, platform: NodeJS.Platform): string {
  const sep = platform === "win32" ? "\\" : "/";
  const trimmed = parent.replace(/[\\/]+$/, "");
  return trimmed ? `${trimmed}${sep}${child}` : `${sep}${child}`;
}

export const expandHome = (folder: string, home: string): string =>
  folder === "~" ? home : folder.replace(/^~(?=[\\/])/, () => home);
