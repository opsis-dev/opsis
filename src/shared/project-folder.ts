const UNSAFE = { word: /[^\w.-]+/g, alnum: /[^a-z0-9]+/gi } as const;

export function safeName(
  text: string,
  {
    fallback,
    lower = true,
    alphabet = "word",
  }: { fallback: string; lower?: boolean; alphabet?: keyof typeof UNSAFE },
): string {
  const safe = text.replace(UNSAFE[alphabet], "-").replace(/^-+|-+$/g, "");
  return (lower ? safe.toLowerCase() : safe) || fallback;
}

export const slugify = (text: string, fallback: string): string => safeName(text, { fallback });

export const projectFolderName = (name: string): string => slugify(name, "demo");

export function joinFolder(parent: string, child: string, platform: string): string {
  const sep = platform === "win32" ? "\\" : "/";
  const trimmed = parent.replace(/[\\/]+$/, "");
  return trimmed ? `${trimmed}${sep}${child}` : `${sep}${child}`;
}

export const expandHome = (folder: string, home: string): string =>
  folder === "~" ? home : folder.replace(/^~(?=[\\/])/, () => home);
