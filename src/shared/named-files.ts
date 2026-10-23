import { isProjectRef, projectRelPath } from "./asset-ref";
import { stringsIn } from "./vars";

const FILE_NAME = /\.[a-z0-9]{2,6}$/i;

export const namedFiles = (params: unknown): string[] =>
  stringsIn(params, (text) => FILE_NAME.test(text.trim()));

export const namedProjectFiles = (params: unknown): string[] =>
  namedFiles(params).filter((value) => projectRelPath(value.trim()) !== null);

export const projectRefsIn = (value: unknown): string[] => [
  ...new Set(stringsIn(value, isProjectRef)),
];
