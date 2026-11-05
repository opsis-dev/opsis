import { execFileSync } from "node:child_process";
import path from "node:path";

const ROOT = path.resolve(import.meta.dirname, "..", "..", "..", "..");

const EDITOR_DATA = path.join(ROOT, "target", "release", "editor-data");

export function buildEditorData(): void {
  return void execFileSync(
    "cargo",
    ["build", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data"],
    {
      cwd: ROOT,
      stdio: "inherit",
    },
  );
}

export function editorData(kind: "catalog" | "film", input?: string): unknown {
  return JSON.parse(
    execFileSync(EDITOR_DATA, [kind], { input, encoding: "utf8", maxBuffer: 1 << 26 }),
  );
}
