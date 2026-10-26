import { preparedPath, preparedSource } from "@opsis/video/common/asset-ref";
import { stringsIn } from "./vars";

const sourceOf = (text: string): string | null => {
  if (text.startsWith("project:")) return text;
  const prepared = preparedPath(text);
  const source = prepared === null ? null : preparedSource(prepared);
  return source?.startsWith("project:") ? source : null;
};

export const projectRefsIn = (value: unknown): string[] => [
  ...new Set(
    stringsIn(value, (text) => sourceOf(text) !== null).flatMap((text) => sourceOf(text) ?? []),
  ),
];
