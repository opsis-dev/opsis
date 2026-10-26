import type { FontData } from "three/addons/loaders/FontLoader.js";

export const FONT_UNREADABLE = "Couldn't prepare this font for 3D text.";

const loaded = new Map<string, Promise<FontData>>();

async function fetchOutlines(url: string): Promise<FontData> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(FONT_UNREADABLE);
  return (await res.json()) as FontData;
}

export function loadOutlines(url: string): Promise<FontData> {
  const key = url;
  let pending = loaded.get(key);
  if (!pending) {
    pending = fetchOutlines(url).catch((error: unknown) => {
      loaded.delete(key);
      throw error instanceof Error && error.message === FONT_UNREADABLE
        ? error
        : new Error(FONT_UNREADABLE, { cause: error });
    });
    loaded.set(key, pending);
  }
  return pending;
}

export function wrapLines(
  text: string,
  maxWidth: number,
  widthOf: (s: string) => number,
): string[] {
  const lines: string[] = [];
  for (const paragraph of text.split("\n")) {
    let line = "";
    for (const word of paragraph.split(/\s+/).filter(Boolean)) {
      const next = line ? `${line} ${word}` : word;
      if (line && widthOf(next) > maxWidth) {
        lines.push(line);
        line = word;
      } else line = next;
    }
    lines.push(line);
  }
  return lines;
}
