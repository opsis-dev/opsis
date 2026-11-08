import { execFileSync } from "node:child_process";
import { mkdirSync, realpathSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { APPIMAGE_TOOLS_CACHE, appImageTools } from "./appimage-tools";
import { hostTarget, stagedFfmpeg } from "./ffmpeg";
import { buildsNatively, ensureStaticQt } from "./static-qt";
import { devCache, ROOT } from "./tool-config";
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

export function cargoProfileVariables(env: NodeJS.ProcessEnv): Record<string, string> {
  return Object.fromEntries(
    Object.entries(env).flatMap(([name, value]) =>
      name.startsWith("CARGO_PROFILE_") && value !== undefined ? [[name, value]] : [],
    ),
  );
}

function zigProgram(): string {
  const found = process.env.ZIG ?? execFileSync("which", ["zig"], { encoding: "utf8" }).trim();
  return realpathSync(found);
}

export type PackageRun = { cargoArgs: string[]; formats: string; out: string };

export const UPDATE_KEY_ENV = ["OPSIS_UPDATE_KEY", "OPSIS_UPDATE_KEY_PASSWORD"];

function commit(): string {
  return (
    process.env.GITHUB_SHA ??
    execFileSync("git", ["rev-parse", "HEAD"], { cwd: ROOT, encoding: "utf8" }).trim()
  );
}

async function toolArgs(run: PackageRun, target: string): Promise<string[]> {
  if (!run.formats.split(",").includes("appimage")) return [];
  const tools = await appImageTools(target);
  return ["--appimagetool", tools.appimagetool, "--appimage-runtime", tools.runtime];
}

function packagerArgs(run: PackageRun, tools: string[]): string[] {
  const target = hostTarget();
  return [
    "--program",
    releaseBinary(),
    "--ffmpeg",
    stagedFfmpeg(target),
    "--target-triple",
    target,
    "--root",
    ROOT,
    "--out",
    run.out,
    "--formats",
    run.formats,
    ...tools,
  ];
}

export async function buildAndPackage(run: PackageRun): Promise<void> {
  const qt = await ensureStaticQt();
  const jobs = process.env.CARGO_BUILD_JOBS ?? "8";
  const tools = await toolArgs(run, hostTarget());
  const build = ["cargo", "build", "--release", "-p", "opsis-ui", ...run.cargoArgs];
  const packager = [
    "cargo",
    "run",
    "-q",
    "-p",
    "opsis-packager",
    "--",
    ...packagerArgs(run, tools),
  ];
  const built = commit();
  mkdirSync(run.out, { recursive: true });
  if (buildsNatively()) {
    const env = { ...process.env, OPSIS_QT: qt, CARGO_BUILD_JOBS: jobs, OPSIS_COMMIT: built };
    execFileSync(build[0], build.slice(1), { cwd: ROOT, stdio: "inherit", env });
    execFileSync(packager[0], packager.slice(1), { cwd: ROOT, stdio: "inherit", env });
    return;
  }
  const cargoHome = process.env.CARGO_HOME ?? path.join(homedir(), ".cargo");
  const rustupHome = process.env.RUSTUP_HOME ?? path.join(homedir(), ".rustup");
  const zig = zigProgram();
  const target = releaseTargetDir();
  runInTrixie({
    script: [`cd ${ROOT}`, build.join(" "), packager.map(quoted).join(" ")].join("\n"),
    mounts: [
      ROOT,
      cargoHome,
      rustupHome,
      path.dirname(zig),
      path.dirname(path.dirname(qt)),
      devCache("qt"),
      APPIMAGE_TOOLS_CACHE,
      run.out,
    ],
    passEnv: UPDATE_KEY_ENV,
    env: {
      OPSIS_COMMIT: built,
      ...cargoProfileVariables(process.env),
      PATH: `${path.join(cargoHome, "bin")}:${SYSTEM_PATH}`,
      CARGO_HOME: cargoHome,
      RUSTUP_HOME: rustupHome,
      CARGO_TARGET_DIR: target,
      CARGO_BUILD_JOBS: jobs,
      OPSIS_QT: qt,
      ZIG: zig,
      ZIG_GLOBAL_CACHE_DIR: path.join(target, "zig-cache"),
      XDG_CACHE_HOME: path.join(target, "cache"),
      APPIMAGE_EXTRACT_AND_RUN: "1",
    },
  });
}

function quoted(arg: string): string {
  return `'${arg.replaceAll("'", "'\\''")}'`;
}
