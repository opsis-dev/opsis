import { readFileSync } from "node:fs";
import { FFMPEG_NOTICE, NOTICES, noticeSection } from "./lib/notices";

const SECTIONS = [FFMPEG_NOTICE];

function sectionLinks(text: string, heading: string): string[] {
  const section = noticeSection(text, heading);
  return [...section.matchAll(/https:\/\/[^\s)`]+/g)].map((found) => found[0]);
}

const text = readFileSync(NOTICES, "utf8");
const links = SECTIONS.flatMap((heading) => sectionLinks(text, heading));
const broken: string[] = [];
for (const link of links) {
  const response = await fetch(link, { redirect: "follow" }).catch(() => null);
  if (!response?.ok) broken.push(`${link} (${response?.status ?? "no answer"})`);
  else console.log(`ok ${link}`);
}
if (broken.length > 0) throw new Error(`Broken links in the source notice:\n${broken.join("\n")}`);
