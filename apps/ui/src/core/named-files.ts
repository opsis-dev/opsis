import { projectRelPath } from "@opsis/video/common/asset-ref";
import { stringsIn } from "./vars";

const FILE_NAME = /\.[a-z0-9]{2,6}$/i;

export const namedProjectFiles = (params: unknown): string[] =>
  stringsIn(params, (text) => FILE_NAME.test(text.trim())).filter(
    (value) => projectRelPath(value.trim()) !== null,
  );

export const projectRefsIn = (value: unknown): string[] => [
  ...new Set(stringsIn(value, (text) => text.startsWith("project:"))),
];
