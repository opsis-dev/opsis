import ts from "typescript";
import { describe, expect, it } from "vitest";
import { NODE_ICON_NAMES } from "./icon-name";

const SCANNED = /\/src\/(shared|renderer)\/(?!.*\.test\.)/;
const LISTS = /\/src\/(shared\/nodes\/icon-name|renderer\/editor\/icons)\.ts$/;
const ICON_NAMES = new Set<string>(NODE_ICON_NAMES);

function parsedConfig(): ts.ParsedCommandLine {
  const host = { ...ts.sys, onUnRecoverableConfigFileDiagnostic: () => undefined };
  const config = ts.getParsedCommandLineOfConfigFile("tsconfig.json", {}, host);
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
  const files = config.fileNames.filter((f) => SCANNED.test(f) && !LISTS.test(f));
  const program = ts.createProgram(files, config.options);
  const checker = program.getTypeChecker();
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
  for (const file of files) {
    const source = program.getSourceFile(file);
    if (source) visit(source);
  }
  return used;
}

describe("node icon names", () => {
  it("lists only icons that a node, card or editor uses", () => {
    const used = iconNamesInUse();
    expect(NODE_ICON_NAMES.filter((name) => !used.has(name))).toEqual([]);
  }, 60_000);
});
