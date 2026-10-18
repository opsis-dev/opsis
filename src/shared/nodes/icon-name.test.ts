import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { NODE_ICON_NAMES } from "./icon-name";

const ROOTS = ["src/shared", "src/renderer"];
const LISTS = new Set(["src/shared/nodes/icon-name.ts", "src/renderer/editor/icons.ts"]);

const sourceText = (): string =>
  ROOTS.flatMap((root) =>
    readdirSync(root, { recursive: true, encoding: "utf8" }).map((file) => path.join(root, file)),
  )
    .filter((file) => /\.tsx?$/.test(file) && !file.includes(".test.") && !LISTS.has(file))
    .map((file) => readFileSync(file, "utf8"))
    .join("\n");

describe("node icon names", () => {
  it("lists only icons that a node, card or editor uses", () => {
    const text = sourceText();
    expect(NODE_ICON_NAMES.filter((name) => !text.includes(`"${name}"`))).toEqual([]);
  });
});
