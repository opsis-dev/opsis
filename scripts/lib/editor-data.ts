import { execFileSync } from "node:child_process";
import path from "node:path";
import { ROOT } from "./tool-config";

const EDITOR_DATA = path.join(ROOT, "target", "release", "editor-data");

export function buildEditorData(): void {
  execFileSync(
    "cargo",
    ["build", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data"],
    {
      cwd: ROOT,
      stdio: "inherit",
    },
  );
}

export function editorCatalog(): unknown {
  return JSON.parse(
    execFileSync(EDITOR_DATA, ["catalog"], { encoding: "utf8", maxBuffer: 1 << 26 }),
  );
}
