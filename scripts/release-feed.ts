import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { z } from "zod";
import { FFMPEG_NOTICE, NOTICES, noticeSection } from "./lib/notices";
import { readJsonWith } from "./lib/read-json";
import { ROOT } from "./lib/tool-config";

const REPOSITORY = "https://github.com/opsis-dev/opsis";
const UPDATE_SUFFIX = ".update.json";
const SELF_UPDATING_KEYS = 4;

const updateFileSchema = z.object({
  key: z.string(),
  file: z.string(),
  signature: z.string(),
});

export type UpdateFile = z.infer<typeof updateFileSchema>;

export function workspaceVersion(manifest: string): string {
  const found = /\[workspace\.package\][^[]*?\nversion = "([^"]+)"/.exec(manifest);
  if (!found?.[1]) throw new Error("The workspace version isn't in Cargo.toml.");
  return found[1];
}

export function releaseNotes(changelog: string, version: string): string {
  const lines = changelog.split("\n");
  const start = lines.findIndex((line) => line.startsWith(`## [${version}]`));
  if (start === -1) throw new Error(`CHANGELOG.md has no section for ${version}.`);
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith("## "));
  return (end === -1 ? rest : rest.slice(0, end)).join("\n").trim();
}

export function githubNotes(notes: string, notices: string): string {
  return `${notes}\n\n${noticeSection(notices, FFMPEG_NOTICE)}\n`;
}

export function feed(version: string, notes: string, files: UpdateFile[], date: string) {
  const platforms = Object.fromEntries(
    files.map((update) => [
      update.key,
      {
        url: `${REPOSITORY}/releases/download/v${version}/${update.file}`,
        signature: update.signature,
      },
    ]),
  );
  if (Object.keys(platforms).length !== files.length) {
    throw new Error("Two update files name the same system.");
  }
  return { version, notes, pub_date: date, platforms };
}

function main(): void {
  const [dir, out, notesOut] = process.argv.slice(2);
  if (!dir || !out || !notesOut) {
    throw new Error("bun scripts/release-feed.ts <artifacts folder> <latest.json> <notes.md>");
  }
  const version = workspaceVersion(readFileSync(path.join(ROOT, "Cargo.toml"), "utf8"));
  const tag = process.env.GITHUB_REF_NAME;
  if (tag && tag !== `v${version}`) throw new Error(`The tag ${tag} isn't v${version}.`);
  const files = readdirSync(dir, { recursive: true, encoding: "utf8" })
    .filter((name) => name.endsWith(UPDATE_SUFFIX))
    .map((name) => readJsonWith(updateFileSchema, path.join(dir, name)));
  if (files.length !== SELF_UPDATING_KEYS) {
    throw new Error(`Expected ${SELF_UPDATING_KEYS} signed update files, found ${files.length}.`);
  }
  const notes = releaseNotes(readFileSync(path.join(ROOT, "CHANGELOG.md"), "utf8"), version);
  const written = feed(version, notes, files, new Date().toISOString());
  writeFileSync(out, `${JSON.stringify(written, null, 2)}\n`);
  writeFileSync(notesOut, githubNotes(notes, readFileSync(NOTICES, "utf8")));
  console.log(`wrote ${out} with ${files.length} update files for ${version}, and ${notesOut}`);
}

if (import.meta.main) main();
