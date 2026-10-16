export const projectFolderName = (name: string): string =>
  name
    .trim()
    .replace(/[^\w.-]+/g, "-")
    .toLowerCase() || "demo";

export function joinFolder(parent: string, child: string, platform: string): string {
  const sep = platform === "win32" ? "\\" : "/";
  const trimmed = parent.replace(/[\\/]+$/, "");
  return `${trimmed || sep}${trimmed ? sep : ""}${child}`;
}
