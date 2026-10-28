import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import { ROOT } from "./lib/tool-config";

const CHECKED = /^(apps|packages|crates|scripts|e2e)\/.*\.(ts|tsx|mts|css|rs)$|^[^/]+\.config\.ts$/;
const SKIPPED = [
  /^apps\/ui\/src\/components\//,
  /^apps\/ui\/src\/routeTree\.gen\.ts$/,
  /\/generated\//,
];
const DIRECTIVE = /^\/\/\/ <reference|^\/[/*]\s*(oxlint-|@ts-)/;
const LINE_COMMENT = /^\s*\/\/|\/\*|\*\//;
const QUOTED_MARK = /"[^"]*(\/\*|\*\/)[^"]*"/;

const scriptComments = (file: string, text: string): number[] => {
  const kind = file.endsWith(".tsx") ? ts.ScriptKind.TSX : ts.ScriptKind.TS;
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, kind);
  const starts = new Set<number>();
  const collect = (pos: number) => {
    for (const range of [
      ...(ts.getLeadingCommentRanges(text, pos) ?? []),
      ...(ts.getTrailingCommentRanges(text, pos) ?? []),
    ]) {
      if (!DIRECTIVE.test(text.slice(range.pos, range.end))) starts.add(range.pos);
    }
  };
  const visit = (node: ts.Node) => {
    if (node.kind === ts.SyntaxKind.JsxText) return;
    const children = node.getChildren(source);
    if (children.length === 0) {
      collect(node.getFullStart());
      collect(node.end);
    }
    children.forEach(visit);
  };
  visit(source);
  return [...starts].map((pos) => source.getLineAndCharacterOfPosition(pos).line);
};

const otherComments = (text: string): number[] =>
  text
    .split("\n")
    .flatMap((line, index) => (LINE_COMMENT.test(line) && !QUOTED_MARK.test(line) ? [index] : []));

const files = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], {
  cwd: ROOT,
  encoding: "utf8",
})
  .split("\n")
  .filter((file) => CHECKED.test(file) && !SKIPPED.some((skip) => skip.test(file)));

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
