import path from "node:path";
import { NODE_ICON_NAMES } from "@opsis/core/nodes/icon-name";
import ts from "typescript";
import { describe, expect, it } from "vitest";

const TSCONFIG = path.join(import.meta.dirname, "..", "..", "tsconfig.json");
const SCANNED = /\/(packages\/core|apps\/ui)\/src\/(?!.*\.test\.)/;
const LISTS = /\/(packages\/core\/src\/nodes\/icon-name|apps\/ui\/src\/editor\/icons)\.ts$/;
const ICON_NAMES = new Set<string>(NODE_ICON_NAMES);

function parsedConfig(): ts.ParsedCommandLine {
  const host = { ...ts.sys, onUnRecoverableConfigFileDiagnostic: () => undefined };
  const config = ts.getParsedCommandLineOfConfigFile(TSCONFIG, {}, host);
  if (!config) throw new Error("tsconfig.json couldn't be read.");
  return config;
}

function takesIconName(type: ts.Type | undefined): boolean {
  if (!type) return false;
  const members = type.isUnion() ? type.types : [type];
  const literals = new Set(members.flatMap((m) => (m.isStringLiteral() ? [m.value] : [])));
  return NODE_ICON_NAMES.every((name) => literals.has(name));
}

function iconNamesInUse(): Set<string> {
  const config = parsedConfig();
  const program = ts.createProgram(config.fileNames, config.options);
  const checker = program.getTypeChecker();
  const files = program
    .getSourceFiles()
    .filter((source) => SCANNED.test(source.fileName) && !LISTS.test(source.fileName));
  const used = new Set<string>();
  const visit = (node: ts.Node) => {
    if (
      ts.isStringLiteralLike(node) &&
      ICON_NAMES.has(node.text) &&
      takesIconName(checker.getContextualType(node))
    )
      used.add(node.text);
    ts.forEachChild(node, visit);
  };
  for (const source of files) visit(source);
  return used;
}

describe("node icon names", () => {
  it("lists only icons that a node, card or editor uses", () => {
    const used = iconNamesInUse();
    expect(NODE_ICON_NAMES.filter((name) => !used.has(name))).toEqual([]);
  }, 60_000);
});
