import { readFileSync } from "node:fs";
import path from "node:path";
import { ROOT } from "./lib/tool-config";

const NOTICES = path.join(ROOT, "THIRD_PARTY_NOTICES.md");

const SECTIONS = ["## FFmpeg"];

function sectionLinks(text: string, heading: string): string[] {
  const start = text.indexOf(`\n${heading}`);
  if (start === -1) throw new Error(`THIRD_PARTY_NOTICES.md has no "${heading}" section.`);
  const end = text.indexOf("\n## ", start + 1);
  const section = text.slice(start, end === -1 ? undefined : end);
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
