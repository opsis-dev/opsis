import { execFileSync, spawnSync } from "node:child_process";
import {
  closeSync,
  existsSync,
  mkdirSync,
  openSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { ROOT } from "./tool-config";
import { runInTrixie } from "./trixie";

export const MACOS_MINIMUM = "14.4";
const BUILD_JOBS = "8";
const COMPLETE = ".opsis-complete";
const LOG_TAIL_LINES = 40;

export type QtPlatform = "linux" | "darwin" | "win32";

export const QT_PLATFORMS: QtPlatform[] = ["linux", "darwin", "win32"];

const SHARED_CONFIGURE = [
  "-static",
  "-release",
  "-no-icu",
  "-no-opengl",
  "-nomake",
  "examples",
  "-nomake",
  "tests",
  "-no-feature-sql",
  "-qt-zlib",
  "-qt-libpng",
  "-qt-libjpeg",
  "-qt-harfbuzz",
  "-qt-pcre",
];

const PLATFORM_CONFIGURE: Record<QtPlatform, string[]> = {
  linux: [
    "-system-freetype",
    "-fontconfig",
    "-xcb",
    "-platform",
    "linux-clang-libc++",
    "--",
    "-DCMAKE_EXE_LINKER_FLAGS=-static-libstdc++",
  ],
  darwin: ["-no-framework", "--", `-DCMAKE_OSX_DEPLOYMENT_TARGET=${MACOS_MINIMUM}`],
  win32: [],
};

export type BuildStep = { dir: string; program: string; args: string[]; log: string };

function isQtPlatform(platform: string): platform is QtPlatform {
  return (QT_PLATFORMS as string[]).includes(platform);
}

function qtPlatform(platform: string = process.platform): QtPlatform {
  if (!isQtPlatform(platform)) throw new Error(`The static Qt isn't built for ${platform}.`);
  return platform;
}

export function configureArgs(platform: QtPlatform): string[] {
  return [...SHARED_CONFIGURE, ...PLATFORM_CONFIGURE[platform]];
}

function script(platform: QtPlatform, name: string): string {
  return platform === "win32" ? `${name}.bat` : name;
}

function cmakeStep(dir: string, args: string[], log: string): BuildStep {
  return { dir, program: "cmake", args, log };
}

export function buildSteps(
  platform: QtPlatform,
  version: string,
  tarballs: string[],
  build: string,
  prefix: string,
): BuildStep[] {
  const [qtbase, qtsvg] = tarballs;
  const join = platform === "win32" ? path.win32.join : path.posix.join;
  const sources = join(build, "src");
  const base = join(build, "qtbase");
  const svg = join(build, "qtsvg");
  return [
    cmakeStep(sources, ["-E", "tar", "xf", qtbase], "unpack-qtbase.log"),
    cmakeStep(sources, ["-E", "tar", "xf", qtsvg], "unpack-qtsvg.log"),
    {
      dir: base,
      program: join(sources, `qtbase-everywhere-src-${version}`, script(platform, "configure")),
      args: ["-prefix", prefix, ...configureArgs(platform)],
      log: "configure.log",
    },
    cmakeStep(base, ["--build", ".", "--parallel", BUILD_JOBS], "build.log"),
    cmakeStep(base, ["--install", "."], "install.log"),
    {
      dir: svg,
      program: join(prefix, "bin", script(platform, "qt-configure-module")),
      args: [join(sources, `qtsvg-everywhere-src-${version}`)],
      log: "configure.log",
    },
    cmakeStep(svg, ["--build", ".", "--parallel", BUILD_JOBS], "build.log"),
    cmakeStep(svg, ["--install", "."], "install.log"),
  ];
}

function quoted(arg: string): string {
  return `'${arg.replaceAll("'", "'\\''")}'`;
}

export function shellScript(steps: BuildStep[], build: string, prefix: string): string {
  const dirs = [...new Set(steps.map((step) => step.dir))];
  return [
    `rm -rf ${quoted(build)} ${quoted(prefix)}`,
    `mkdir -p ${dirs.map(quoted).join(" ")}`,
    ...steps.map(
      (step) =>
        `cd ${quoted(step.dir)} && ${[step.program, ...step.args].map(quoted).join(" ")} > ${quoted(step.log)}`,
    ),
    `rm -rf ${quoted(build)}`,
    `touch ${quoted(path.posix.join(prefix, COMPLETE))}`,
  ].join("\n");
}

function logTail(file: string): string {
  if (!existsSync(file)) return "";
  return readFileSync(file, "utf8").split("\n").slice(-LOG_TAIL_LINES).join("\n");
}

function runStep(platform: QtPlatform, step: BuildStep, env: NodeJS.ProcessEnv): void {
  const log = path.join(step.dir, step.log);
  const [program, args] = step.program.endsWith(".bat")
    ? ["cmd.exe", ["/d", "/s", "/c", step.program, ...step.args]]
    : [step.program, step.args];
  const out = openSync(log, "w");
  try {
    execFileSync(program, args, { cwd: step.dir, env, stdio: ["ignore", out, "inherit"] });
  } catch (error) {
    console.error(logTail(log));
    throw new Error(`${path.basename(step.program)} failed on ${platform}; its log is ${log}`, {
      cause: error,
    });
  } finally {
    closeSync(out);
  }
}

export function parseSetOutput(text: string): Record<string, string> {
  const entries: [string, string][] = [];
  for (const line of text.split(/\r?\n/)) {
    const at = line.indexOf("=");
    if (at > 0) entries.push([line.slice(0, at), line.slice(at + 1)]);
  }
  return Object.fromEntries(entries);
}

function msvcEnv(): NodeJS.ProcessEnv {
  if (process.env.VCINSTALLDIR) return process.env;
  const programs = process.env["ProgramFiles(x86)"] ?? "C:\\Program Files (x86)";
  const vswhere = path.win32.join(programs, "Microsoft Visual Studio", "Installer", "vswhere.exe");
  if (!existsSync(vswhere))
    throw new Error(
      `Building Qt on Windows needs Visual Studio's C++ tools; ${vswhere} is missing.`,
    );
  const install = execFileSync(
    vswhere,
    [
      "-latest",
      "-products",
      "*",
      "-requires",
      "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
      "-property",
      "installationPath",
    ],
    { encoding: "utf8" },
  ).trim();
  if (!install) throw new Error("Visual Studio has no C++ tools for x64 installed.");
  const vcvars = path.win32.join(install, "VC", "Auxiliary", "Build", "vcvars64.bat");
  const set = spawnSync("cmd.exe", ["/d", "/s", "/c", `""${vcvars}" >nul && set"`], {
    encoding: "utf8",
    windowsVerbatimArguments: true,
  });
  if (set.status !== 0) throw new Error(`${vcvars} failed: ${set.stderr}`);
  return { ...process.env, ...parseSetOutput(set.stdout) };
}

function buildEnv(platform: QtPlatform): NodeJS.ProcessEnv {
  if (platform === "darwin") return { ...process.env, MACOSX_DEPLOYMENT_TARGET: MACOS_MINIMUM };
  if (platform === "win32") return msvcEnv();
  return process.env;
}

function runNatively(platform: QtPlatform, steps: BuildStep[], build: string, prefix: string) {
  const env = buildEnv(platform);
  rmSync(build, { recursive: true, force: true });
  rmSync(prefix, { recursive: true, force: true });
  for (const dir of new Set(steps.map((step) => step.dir))) mkdirSync(dir, { recursive: true });
  for (const step of steps) runStep(platform, step, env);
  rmSync(build, { recursive: true, force: true });
  writeFileSync(path.join(prefix, COMPLETE), "");
}

function buildDir(platform: QtPlatform, dir: string): string {
  const runnerTemp = process.env.RUNNER_TEMP;
  return platform === "win32" && runnerTemp
    ? path.join(runnerTemp, "qtb")
    : path.join(dir, "build");
}

export function buildsNatively(): boolean {
  return process.platform !== "linux";
}

export type StaticQtBuild = {
  version: string;
  tarballs: string[];
  dir: string;
  mounts: string[];
};

export function buildStaticQt(run: StaticQtBuild): string {
  const platform = qtPlatform();
  const prefix = path.join(run.dir, "prefix");
  const build = buildDir(platform, run.dir);
  const steps = buildSteps(platform, run.version, run.tarballs, build, prefix);
  if (buildsNatively()) runNatively(platform, steps, build, prefix);
  else
    runInTrixie({
      script: shellScript(steps, build, prefix),
      mounts: [run.dir, ROOT, ...run.mounts],
    });
  if (!existsSync(path.join(prefix, COMPLETE)))
    throw new Error(`The static Qt build didn't finish in ${prefix}.`);
  if (!existsSync(path.join(prefix, "include", "QtSvg")))
    throw new Error(`The static Qt build in ${prefix} has no Qt SVG.`);
  return prefix;
}
