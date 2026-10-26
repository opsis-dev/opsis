import { textRules } from "@/store/catalog";

const varPattern = () => new RegExp(textRules().variablePattern, "g");

export const pauseMark = (): string => textRules().pauseMark;

export type TextToken =
  | { kind: "text"; text: string }
  | { kind: "var"; name: string }
  | { kind: "pause" };

const plainText = (text: string): TextToken[] => (text ? [{ kind: "text", text }] : []);

function textWithPauses(text: string): TextToken[] {
  const out: TextToken[] = [];
  for (const [i, part] of text.split(pauseMark()).entries()) {
    if (i > 0) out.push({ kind: "pause" });
    out.push(...plainText(part));
  }
  return out;
}

export function tokenize(value: string, pauses = false): TextToken[] {
  const text = pauses ? textWithPauses : plainText;
  const out: TextToken[] = [];
  let last = 0;
  for (const m of value.matchAll(varPattern())) {
    out.push(...text(value.slice(last, m.index)), { kind: "var", name: m[1] });
    last = m.index + m[0].length;
  }
  out.push(...text(value.slice(last)));
  return out;
}

export const varToken = (name: string): string => `{{${name}}}`;
