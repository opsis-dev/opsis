import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync } from "node:fs";

function git(args: string[], cwd: string) {
  execFileSync("git", ["-c", "core.longpaths=true", ...args], { cwd, stdio: "inherit" });
}

export function fetchGitSource(
  src: string,
  marker: string,
  source: { repo: string; commit: string },
  label: string,
): void {
  if (existsSync(marker)) {
    console.log(`${label}: ${src} is already fetched`);
    return;
  }
  mkdirSync(src, { recursive: true });
  git(["init", "-q"], src);
  git(["remote", "add", "origin", source.repo], src);
  git(["fetch", "-q", "--depth", "1", "origin", source.commit], src);
  git(["checkout", "-q", "FETCH_HEAD"], src);
  console.log(`${label}: ${source.commit}`);
}
