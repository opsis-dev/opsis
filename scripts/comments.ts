import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { parseSync } from "oxc-parser";
import { ROOT } from "./lib/tool-config";

const CHECKED =
  /^(apps|packages|crates|scripts|e2e)\/.*\.(ts|tsx|mts|css|rs|zig|zon)$|^[^/]+\.config\.ts$/;
const SKIPPED = [
  /^apps\/frontend\/src\/components\//,
  /^apps\/frontend\/src\/routeTree\.gen\.ts$/,
  /\/generated\//,
];
const DIRECTIVE = /^\/\/\/ <reference|^\/[/*]\s*(oxlint-|@ts-)/;
const LINE_COMMENT = /^\s*\/\/|\/\*|\*\//;
const QUOTED_MARK = /"[^"]*(\/\*|\*\/)[^"]*"/;

function lineOf(text: string, pos: number): number {
  return text.slice(0, pos).split("\n").length - 1;
}

function scriptComments(file: string, text: string): number[] {
  const { comments, errors } = parseSync(file, text);
  const [error] = errors;
  if (error) throw new Error(`${file} doesn't parse: ${error.message}`);
  return comments
    .filter((comment) => !DIRECTIVE.test(text.slice(comment.start, comment.end)))
    .map((comment) => lineOf(text, comment.start));
}

function otherComments(text: string): number[] {
  return text
    .split("\n")
    .flatMap((line, index) => (LINE_COMMENT.test(line) && !QUOTED_MARK.test(line) ? [index] : []));
}

const files = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], {
  cwd: ROOT,
  encoding: "utf8",
})
  .split("\n")
  .filter((file) => CHECKED.test(file) && !SKIPPED.some((skip) => skip.test(file)))
  .filter((file) => existsSync(path.join(ROOT, file)));

const found = files.flatMap((file) => {
  const text = readFileSync(path.join(ROOT, file), "utf8");
  const lines = /\.(ts|tsx|mts)$/.test(file) ? scriptComments(file, text) : otherComments(text);
  const all = text.split("\n");
  return lines.toSorted((a, b) => a - b).map((line) => `${file}:${line + 1}: ${all[line]?.trim()}`);
});

if (found.length > 0) {
  console.error(found.join("\n"));
  console.error(`${found.length} comments. Opsis has none; see AGENTS.md.`);
  process.exit(1);
}
console.log(`No comments in ${files.length} files.`);
