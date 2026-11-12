import { execFileSync } from "node:child_process";
import path from "node:path";
import { stageFfmpeg } from "./lib/ffmpeg";
import { ROOT, devCache } from "./lib/tool-config";

const ORACLE_DIR = devCache("oracle");
const BUILD_JOBS = "8";
const DIRTY_FLAG = "--dirty";

function git(args: string[]): string {
  return execFileSync("git", args, { cwd: ROOT, encoding: "utf8" }).trim();
}

function run(command: string, args: string[], env: NodeJS.ProcessEnv = process.env) {
  execFileSync(command, args, { cwd: ROOT, stdio: "inherit", env });
}

const args = process.argv.slice(2);
const dirtyAllowed = args.includes(DIRTY_FLAG);
const [part = "all", only = ""] = args.filter((a) => a !== DIRTY_FLAG);
const commit = git(["rev-parse", "HEAD"]);
const dirty = git(["status", "--porcelain"]) !== "";
if (dirty && !dirtyAllowed) {
  console.error(
    `The working tree has changes, so its pictures wouldn't belong to ${commit}. Commit first, or pass ${DIRTY_FLAG} for a trial run.`,
  );
  process.exit(1);
}
const out = path.join(ORACLE_DIR, dirty ? `${commit}-dirty` : commit);
run("bun", ["run", "film:bundle"]);
run("bun", ["run", "oracle:bundle"]);
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
  {
    ...process.env,
    CARGO_TARGET_DIR: path.join(ORACLE_DIR, "target"),
    CARGO_BUILD_JOBS: BUILD_JOBS,
    OPSIS_ORACLE_COMMIT: dirty ? `${commit}-dirty` : commit,
  },
);
