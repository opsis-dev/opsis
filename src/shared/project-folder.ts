export const slugify = (text: string, fallback: string): string =>
  text
    .replace(/[^\w.-]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .toLowerCase() || fallback;

export const projectFolderName = (name: string): string => slugify(name, "demo");

export function joinFolder(parent: string, child: string, platform: string): string {
  const sep = platform === "win32" ? "\\" : "/";
  const trimmed = parent.replace(/[\\/]+$/, "");
  return trimmed ? `${trimmed}${sep}${child}` : `${sep}${child}`;
}

export const expandHome = (folder: string, home: string): string =>
  folder === "~" ? home : folder.replace(/^~(?=[\\/])/, () => home);
