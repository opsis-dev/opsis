import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { stageFfmpeg } from "./lib/ffmpeg";
import { ROOT, devCache } from "./lib/tool-config";

const ORACLE_DIR = devCache("oracle");
const SOURCE_DIR = path.join(ORACLE_DIR, "src");
const BUILD_JOBS = "8";
const INTO_FLAG = "--into=";
const COMMIT_FLAG = "--commit=";

function git(args: string[], cwd = ROOT): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
}

function run(command: string, args: string[], cwd: string, env: NodeJS.ProcessEnv = process.env) {
  execFileSync(command, args, { cwd, stdio: "inherit", env });
}

function flag(args: string[], prefix: string): string | undefined {
  return args.find((a) => a.startsWith(prefix))?.slice(prefix.length);
}

function checkout(commit: string): string {
  if (!existsSync(path.join(SOURCE_DIR, ".git"))) {
    run("git", ["worktree", "add", "--detach", SOURCE_DIR, commit], ROOT);
  } else {
    run("git", ["checkout", "--quiet", "--detach", commit], SOURCE_DIR);
  }
  return SOURCE_DIR;
}

const args = process.argv.slice(2);
const into = flag(args, INTO_FLAG);
const commit = git(["rev-parse", flag(args, COMMIT_FLAG) ?? "HEAD"]);
const [part = "all", only = ""] = args.filter((a) => !a.startsWith("--"));
const out = path.join(ORACLE_DIR, into ?? commit);
if (into && !existsSync(path.join(out, "oracle.json"))) {
  console.error(`There's no oracle in ${out} to add ${part} to.`);
  process.exit(1);
}
const source = checkout(commit);
run("bun", ["install", "--frozen-lockfile"], source);
run("bun", ["run", "film:bundle"], source);
run("bun", ["run", "oracle:bundle"], source);
const ffmpeg = await stageFfmpeg();
run(
  "cargo",
  [
    "run",
    "-q",
    "--release",
    "-p",
    "opsis-oracle",
    "--bin",
    "oracle",
    "--",
    out,
    ffmpeg,
    part,
    only,
  ],
  source,
  {
    ...process.env,
    CARGO_TARGET_DIR: path.join(ORACLE_DIR, "target"),
    CARGO_BUILD_JOBS: BUILD_JOBS,
    OPSIS_ORACLE_COMMIT: commit,
  },
);
