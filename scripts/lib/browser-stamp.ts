import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, statSync } from "node:fs";

export type CheckedBrowser = {
  path: string;
  version: string;
  sha256: string;
  size: number;
  modifiedMs: number;
};

const VERSION = /\b\d+\.\d+\.\d+\.\d+\b/;

export function checkedBrowser(file: string): CheckedBrowser {
  const said = execFileSync(file, ["--version"], { encoding: "utf8" });
  const version = VERSION.exec(said)?.[0];
  if (!version) throw new Error(`${file} didn't say its version: ${said.trim()}`);
  const stat = statSync(file);
  return {
    path: file,
    version,
    sha256: createHash("sha256").update(readFileSync(file)).digest("hex"),
    size: stat.size,
    modifiedMs: Math.floor(stat.mtimeMs),
  };
}
