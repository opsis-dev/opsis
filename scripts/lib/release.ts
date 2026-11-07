import { execFileSync } from "node:child_process";
import { realpathSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { buildsNatively, ensureStaticQt } from "./static-qt";
import { ROOT } from "./tool-config";
import { runInTrixie } from "./trixie";

const SYSTEM_PATH = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

export function releaseTargetDir(): string {
  return buildsNatively() || process.platform !== "linux"
    ? path.join(ROOT, "target")
    : path.join(ROOT, "target", "trixie");
}

export function releaseBinary(): string {
  const exe = process.platform === "win32" ? ".exe" : "";
  return path.join(releaseTargetDir(), "release", `opsis${exe}`);
}

function zigProgram(): string {
  const found = process.env.ZIG ?? execFileSync("which", ["zig"], { encoding: "utf8" }).trim();
  return realpathSync(found);
}

export async function buildRelease(cargoArgs: string[]): Promise<string> {
  const qt = await ensureStaticQt();
  const jobs = process.env.CARGO_BUILD_JOBS ?? "8";
  const build = ["cargo", "build", "--release", "-p", "opsis-ui", ...cargoArgs];
  if (buildsNatively()) {
    execFileSync(build[0], build.slice(1), {
      cwd: ROOT,
      stdio: "inherit",
      env: { ...process.env, OPSIS_QT: qt, CARGO_BUILD_JOBS: jobs },
    });
    return releaseBinary();
  }
  const cargoHome = process.env.CARGO_HOME ?? path.join(homedir(), ".cargo");
  const rustupHome = process.env.RUSTUP_HOME ?? path.join(homedir(), ".rustup");
  const zig = zigProgram();
  const target = releaseTargetDir();
  runInTrixie({
    script: `cd ${ROOT}\n${build.join(" ")}`,
    mounts: [
      ROOT,
      cargoHome,
      rustupHome,
      path.dirname(zig),
      path.dirname(path.dirname(qt)),
      path.join(homedir(), ".cache", "opsis-qt"),
    ],
    env: {
      PATH: `${path.join(cargoHome, "bin")}:${SYSTEM_PATH}`,
      CARGO_HOME: cargoHome,
      RUSTUP_HOME: rustupHome,
      CARGO_TARGET_DIR: target,
      CARGO_BUILD_JOBS: jobs,
      OPSIS_QT: qt,
      ZIG: zig,
      ZIG_GLOBAL_CACHE_DIR: path.join(target, "zig-cache"),
      XDG_CACHE_HOME: path.join(target, "cache"),
    },
  });
  return releaseBinary();
}
