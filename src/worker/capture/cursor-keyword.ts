export const DEFAULT_CURSOR = "default";

const URL_VALUE = /url\(\s*(?:"[^"]*"|'[^']*'|[^)]*)\s*\)(?:\s+[-+\d.]+){0,2}/gi;
const KEYWORD = /^[a-z][a-z-]*$/;

export type CursorUnder = { cursor: string; editable: boolean };

export function cursorKeyword({ cursor, editable }: CursorUnder): string {
  const keyword =
    cursor
      .replace(URL_VALUE, "")
      .split(",")
      .map((part) => part.trim().toLowerCase())
      .findLast(Boolean) ?? "auto";
  if (!KEYWORD.test(keyword)) return DEFAULT_CURSOR;
  if (keyword === "auto") return editable ? "text" : DEFAULT_CURSOR;
  return keyword;
}
