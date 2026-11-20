import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import path from "node:path";
import { appImageTools } from "./appimage-tools";
import { hostTarget, stagedFfmpeg } from "./ffmpeg";
import { readPaths } from "./setup/store";
import { MACOS_MINIMUM } from "./static-qt";
import { ROOT } from "./tool-config";

export function releaseBinary(name = "opsis"): string {
  const exe = process.platform === "win32" ? ".exe" : "";
  return path.join(ROOT, "target", "release", `${name}${exe}`);
}

export const RENDERER_PROGRAM = "opsis-renderer";
export const MCP_PROGRAM = "opsis-mcp";

export type PackageRun = { cargoArgs: string[]; formats: string; out: string };

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
    stagedFfmpeg(),
    "--renderer",
    releaseBinary(RENDERER_PROGRAM),
    "--mcp",
    releaseBinary(MCP_PROGRAM),
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

function platformVariables(): Record<string, string> {
  if (process.platform === "darwin") return { MACOSX_DEPLOYMENT_TARGET: MACOS_MINIMUM };
  if (process.platform === "linux") return { APPIMAGE_EXTRACT_AND_RUN: "1" };
  return {};
}

export async function buildAndPackage(run: PackageRun): Promise<void> {
  const tools = await toolArgs(run, hostTarget());
  const build = [
    "build",
    "--release",
    "-p",
    "opsis-ui",
    "-p",
    "opsis-effects",
    "-p",
    "opsis-mcp",
    ...run.cargoArgs,
  ];
  const packager = ["run", "-q", "-p", "opsis-packager", "--", ...packagerArgs(run, tools)];
  const env = {
    ...process.env,
    OPSIS_QT: readPaths().qt,
    CARGO_BUILD_JOBS: process.env.CARGO_BUILD_JOBS ?? "8",
    OPSIS_COMMIT: commit(),
    ...platformVariables(),
  };
  mkdirSync(run.out, { recursive: true });
  execFileSync("cargo", build, { cwd: ROOT, stdio: "inherit", env });
  execFileSync("cargo", packager, { cwd: ROOT, stdio: "inherit", env });
}
