import { staticFile } from "remotion";

export function publicFile(path: string): string {
  return staticFile(path);
}
