export const VAR = /\{\{\s*([^{}]+?)\s*\}\}/g;

export const PAUSE_MARK = "¦";

export type TextToken =
  | { kind: "text"; text: string }
  | { kind: "var"; name: string }
  | { kind: "pause" };

const plainText = (text: string): TextToken[] => (text ? [{ kind: "text", text }] : []);

const textWithPauses = (text: string): TextToken[] =>
  text
    .split(PAUSE_MARK)
    .flatMap((part, i): TextToken[] => [
      ...(i ? [{ kind: "pause" } as const] : []),
      ...plainText(part),
    ]);

export function tokenize(value: string, pauses = false): TextToken[] {
  const text = pauses ? textWithPauses : plainText;
  const out: TextToken[] = [];
  let last = 0;
  for (const m of value.matchAll(VAR)) {
    out.push(...text(value.slice(last, m.index)), { kind: "var", name: m[1] });
    last = m.index + m[0].length;
  }
  out.push(...text(value.slice(last)));
  return out;
}

function walkStrings(value: unknown, fn: (text: string) => string): unknown {
  if (typeof value === "string") return fn(value);
  if (Array.isArray(value)) return value.map((v) => walkStrings(v, fn));
  if (value && typeof value === "object")
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, walkStrings(v, fn)]));
  return value;
}

const mapStrings = <T>(value: T, fn: (text: string) => string): T => walkStrings(value, fn) as T;

export const interpolate = <T>(value: T, vars: Record<string, string>): T =>
  mapStrings(value, (text) =>
    text.replace(VAR, (whole, name: string) => (Object.hasOwn(vars, name) ? vars[name] : whole)),
  );

export const renameVariable = <T>(value: T, from: string, to: string): T =>
  mapStrings(value, (text) =>
    text.replace(VAR, (whole, name: string) => (name === from ? `{{${to}}}` : whole)),
  );
