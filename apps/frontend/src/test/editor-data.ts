import { execFileSync } from "node:child_process";
import path from "node:path";

const ROOT = path.resolve(import.meta.dirname, "..", "..", "..", "..");

export const EDITOR_DATA = path.join(ROOT, "target", "release", "editor-data");

export const buildEditorData = (): void =>
  void execFileSync(
    "cargo",
    ["build", "-q", "--release", "-p", "opsis-compile", "--bin", "editor-data"],
    {
      cwd: ROOT,
      stdio: "inherit",
    },
  );

export const editorData = (kind: "catalog" | "film", input?: string): unknown =>
  JSON.parse(execFileSync(EDITOR_DATA, [kind], { input, encoding: "utf8", maxBuffer: 1 << 26 }));
