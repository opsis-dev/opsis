import path from "node:path";
import { ROOT } from "./tool-config";

export const NOTICES = path.join(ROOT, "THIRD_PARTY_NOTICES.md");
export const FFMPEG_NOTICE = "## FFmpeg";

export function noticeSection(text: string, heading: string): string {
  const start = text.indexOf(`\n${heading}`);
  if (start === -1) throw new Error(`THIRD_PARTY_NOTICES.md has no "${heading}" section.`);
  const end = text.indexOf("\n## ", start + 1);
  return text.slice(start + 1, end === -1 ? undefined : end).trim();
}
