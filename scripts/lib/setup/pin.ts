import { execFileSync } from "node:child_process";
import { z } from "zod";
import { fetchOk } from "../download";
import { type Pinned, pinned } from "../pins";
import { type Pins, platformName, readPins, writePins } from "./manifest";
import { downloadsDir } from "./store";

const ZIG_INDEX = "https://ziglang.org/download/index.json";
const ZIG_TARGETS: Record<string, string> = {
  "x86_64-linux": platformName("linux", "x64"),
  "aarch64-linux": platformName("linux", "arm64"),
  "x86_64-macos": platformName("darwin", "x64"),
  "aarch64-macos": platformName("darwin", "arm64"),
  "x86_64-windows": platformName("win32", "x64"),
};
const QT_MODULES = ["qtbase", "qtsvg"];
const KDDW_REPOSITORY = "KDAB/KDDockWidgets";
const FILAMENT_REPOSITORY = "https://github.com/google/filament.git";
const FFMPEG_REPOSITORY = "eugeneware/ffmpeg-static";
const FFMPEG_TARGETS = [
  ["x86_64-unknown-linux-gnu", "linux-x64"],
  ["aarch64-unknown-linux-gnu", "linux-arm64"],
  ["x86_64-apple-darwin", "darwin-x64"],
  ["aarch64-apple-darwin", "darwin-arm64"],
  ["x86_64-pc-windows-msvc", "win32-x64"],
] as const;
const NPM_REGISTRY = "https://registry.npmjs.org";

const zigIndexSchema = z.record(z.string(), z.record(z.string(), z.unknown()));
const zigArchiveSchema = z.object({
  tarball: z.string(),
  shasum: z.string(),
  size: z.union([z.string(), z.number()]),
});
const npmVersionSchema = z.object({ version: z.string(), dist: z.object({ tarball: z.string() }) });

async function pinZig(pins: Pins, version: string): Promise<void> {
  const index = zigIndexSchema.parse(await (await fetchOk(ZIG_INDEX)).json());
  const release = index[version];
  if (!release) throw new Error(`${ZIG_INDEX} has no Zig ${version}.`);
  const archives: Record<string, Pinned> = {};
  for (const [target, platform] of Object.entries(ZIG_TARGETS)) {
    const archive = zigArchiveSchema.parse(release[target]);
    archives[platform] = {
      url: archive.tarball,
      sha256: archive.shasum,
      size: Number(archive.size),
    };
  }
  pins.zig = { version, archives };
}

async function pinQt(pins: Pins, version: string): Promise<void> {
  const minor = version.split(".").slice(0, 2).join(".");
  const sources = [];
  for (const module of QT_MODULES) {
    const url = `https://download.qt.io/official_releases/qt/${minor}/${version}/submodules/${module}-everywhere-src-${version}.tar.xz`;
    sources.push({ module, ...(await pinned(url, downloadsDir())) });
  }
  pins.qt = { version, sources };
}

async function pinKddw(pins: Pins, version: string): Promise<void> {
  const url = `https://github.com/${KDDW_REPOSITORY}/releases/download/v${version}/kddockwidgets-${version}.tar.gz`;
  pins.kddw = { version, ...(await pinned(url, downloadsDir())) };
}

export function tagCommit(listed: string, tag: string): string | null {
  const lines = listed.split("\n").map((line) => line.trim().split(/\s+/));
  const peeled = lines.find(([, ref]) => ref === `refs/tags/${tag}^{}`);
  const plain = lines.find(([, ref]) => ref === `refs/tags/${tag}`);
  return (peeled ?? plain)?.[0] ?? null;
}

export function pinFilament(pins: Pins, version: string): void {
  const listed = execFileSync(
    "git",
    ["ls-remote", FILAMENT_REPOSITORY, `refs/tags/${version}`, `refs/tags/${version}^{}`],
    { encoding: "utf8" },
  );
  const commit = tagCommit(listed, version);
  if (!commit) throw new Error(`${FILAMENT_REPOSITORY} has no tag ${version}.`);
  pins.filament = { repo: FILAMENT_REPOSITORY, version, commit };
}

async function pinFfmpeg(pins: Pins, release: string): Promise<void> {
  function url(file: string) {
    return `https://github.com/${FFMPEG_REPOSITORY}/releases/download/${release}/${file}`;
  }
  const targets = [];
  for (const [target, asset] of FFMPEG_TARGETS)
    targets.push({
      target,
      asset,
      program: await pinned(url(`ffmpeg-${asset}.gz`), downloadsDir()),
      probe: await pinned(url(`ffprobe-${asset}.gz`), downloadsDir()),
      licence: await pinned(url(`${asset}.LICENSE`), downloadsDir()),
    });
  pins.ffmpeg = { release, targets };
}

async function pinNpm(pins: Pins, spec: string): Promise<void> {
  const at = spec.lastIndexOf("@");
  const [name, version] = at > 0 ? [spec.slice(0, at), spec.slice(at + 1)] : [spec, "latest"];
  const found = npmVersionSchema.parse(
    await (await fetchOk(`${NPM_REGISTRY}/${name}/${version}`)).json(),
  );
  pins.npm[name] = {
    version: found.version,
    ...(await pinned(found.dist.tarball, downloadsDir())),
  };
}

const PINNERS: Record<string, (pins: Pins, value: string) => Promise<void> | void> = {
  zig: pinZig,
  qt: pinQt,
  kddw: pinKddw,
  filament: pinFilament,
  ffmpeg: pinFfmpeg,
  npm: pinNpm,
  skia: (pins, commit) => {
    pins.skia = { ...pins.skia, commit };
  },
};

export async function pin(args: string[]): Promise<void> {
  const [item, value] = args;
  const pinner = item ? PINNERS[item] : undefined;
  if (!pinner || !value)
    throw new Error(
      `usage: bun run setup pin <${Object.keys(PINNERS).join("|")}> <version, commit or npm name@version>`,
    );
  const pins = readPins();
  await pinner(pins, value);
  writePins(pins);
}
