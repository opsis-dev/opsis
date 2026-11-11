import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
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
import { devCache } from "./tool-config";
import { runInTrixie } from "./trixie";

export const QT_VERSION = "6.12.0";
export const MACOS_MINIMUM = "14.4";
const BUILD_JOBS = "8";
const COMPLETE = ".opsis-complete";
const LOG_TAIL_LINES = 40;
const SOURCES = [
  {
    module: "qtbase",
    sha256: "a951bd163c7b80fc6b8c88d7668fb56abf91c152373e13c10666763238131307",
    size: 51220268,
  },
  {
    module: "qtsvg",
    sha256: "e4ab39534ec97987b1b9b60ef7f4d3253d5912a69f8c713a01753174a4029331",
    size: 2341876,
  },
];

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
  linux: ["-system-freetype", "-fontconfig", "-xcb"],
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

function cacheDir(): string {
  return devCache("qt-static");
}

export function qtFolder(platform: QtPlatform, arch: string = process.arch): string {
  if (platform === "darwin") return "macos";
  if (platform === "win32") return "msvc2022_64";
  return arch === "arm64" ? "gcc_arm64" : "gcc_64";
}

export function staticQtPrefix(): string {
  return path.join(cacheDir(), QT_VERSION, qtFolder(qtPlatform()));
}

export function staticQtReady(): boolean {
  return existsSync(path.join(staticQtPrefix(), COMPLETE));
}

function tarballName(module: string): string {
  return `${module}-everywhere-src-${QT_VERSION}.tar.xz`;
}

function tarballUrl(module: string): string {
  const minor = QT_VERSION.split(".").slice(0, 2).join(".");
  return `https://download.qt.io/official_releases/qt/${minor}/${QT_VERSION}/submodules/${tarballName(module)}`;
}

function sha256(bytes: Uint8Array): string {
  return createHash("sha256").update(bytes).digest("hex");
}

async function download(source: (typeof SOURCES)[number]): Promise<string> {
  const downloads = path.join(cacheDir(), "downloads");
  mkdirSync(downloads, { recursive: true });
  const tarball = path.join(downloads, tarballName(source.module));
  if (!existsSync(tarball) || readFileSync(tarball).length !== source.size) {
    const response = await fetch(tarballUrl(source.module));
    if (!response.ok)
      throw new Error(`downloading ${tarballUrl(source.module)}: ${response.status}`);
    writeFileSync(tarball, new Uint8Array(await response.arrayBuffer()));
  }
  const bytes = readFileSync(tarball);
  if (bytes.length !== source.size || sha256(bytes) !== source.sha256) {
    rmSync(tarball);
    throw new Error(`${tarball} isn't the pinned ${source.module} ${QT_VERSION}; it was removed`);
  }
  return tarball;
}

function script(platform: QtPlatform, name: string): string {
  return platform === "win32" ? `${name}.bat` : name;
}

function cmakeStep(dir: string, args: string[], log: string): BuildStep {
  return { dir, program: "cmake", args, log };
}

export function buildSteps(
  platform: QtPlatform,
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
      program: join(sources, `qtbase-everywhere-src-${QT_VERSION}`, script(platform, "configure")),
      args: ["-prefix", prefix, ...configureArgs(platform)],
      log: "configure.log",
    },
    cmakeStep(base, ["--build", ".", "--parallel", BUILD_JOBS], "build.log"),
    cmakeStep(base, ["--install", "."], "install.log"),
    {
      dir: svg,
      program: join(prefix, "bin", script(platform, "qt-configure-module")),
      args: [join(sources, `qtsvg-everywhere-src-${QT_VERSION}`)],
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

function buildDir(platform: QtPlatform): string {
  const runnerTemp = process.env.RUNNER_TEMP;
  return platform === "win32" && runnerTemp
    ? path.join(runnerTemp, "qtb")
    : path.join(cacheDir(), "build");
}

export function buildsNatively(): boolean {
  return process.platform !== "linux";
}

export async function ensureStaticQt(): Promise<string> {
  const platform = qtPlatform();
  const prefix = staticQtPrefix();
  if (staticQtReady()) return prefix;
  const tarballs = await Promise.all(SOURCES.map(download));
  const build = buildDir(platform);
  const steps = buildSteps(platform, tarballs, build, prefix);
  if (buildsNatively()) runNatively(platform, steps, build, prefix);
  else runInTrixie({ script: shellScript(steps, build, prefix), mounts: [cacheDir()] });
  if (!staticQtReady()) throw new Error(`The static Qt build didn't finish in ${prefix}.`);
  return prefix;
}
