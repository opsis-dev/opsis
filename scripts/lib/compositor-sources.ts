import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { z } from "zod";
import { ROOT } from "./tool-config";

export const LICENCES = path.join(ROOT, "licences");
export const GPL_2 = path.join(LICENCES, "GPL-2.0.txt");
const MANIFEST = path.join(LICENCES, "compositor-sources.json");
const RELEASES = "https://github.com/bismitpanda/opsis/releases/tag";

const manifestSchema = z.object({
  remotion: z.string(),
  ffmpeg: z.string(),
  chain: z.object({
    compositor: z.string(),
    rustFfmpeg: z.string(),
    rustFfmpegSys: z.string(),
    splitter: z.string(),
  }),
  sources: z.array(z.object({ name: z.string(), what: z.string(), url: z.string() })),
});

export type CompositorSources = z.infer<typeof manifestSchema>;

export function readCompositorSources(): CompositorSources {
  return manifestSchema.parse(JSON.parse(readFileSync(MANIFEST, "utf8")));
}

export function installedRemotion(): string {
  const fromSidecar = createRequire(path.join(ROOT, "apps", "sidecar", "package.json"));
  const pkg = z
    .object({ version: z.string() })
    .parse(
      JSON.parse(readFileSync(fromSidecar.resolve("@remotion/renderer/package.json"), "utf8")),
    );
  return pkg.version;
}

export function appVersion(): string {
  const cargo = readFileSync(path.join(ROOT, "Cargo.toml"), "utf8");
  const version = /\[workspace\.package\][^[]*?\bversion\s*=\s*"([^"]+)"/.exec(cargo)?.[1];
  if (!version) throw new Error("Cargo.toml names no workspace version.");
  return version;
}

export function assetName(url: string): string {
  return decodeURIComponent(new URL(url).pathname.split("/").filter(Boolean).at(-1) ?? "source");
}

export function sourcesNote(manifest: CompositorSources, release: string): string {
  const lines = [
    "# FFmpeg in the video renderer",
    "",
    `Opsis ships Remotion ${manifest.remotion}'s compositor, which includes FFmpeg (${manifest.ffmpeg}) built with x264, x265, libaom, dav1d, fdk-aac-free, Opus, libvpx, LAME and zimg. That FFmpeg build is licensed under the GNU General Public License, version 2 or (at your option) any later version; the licence is in COPYING.GPLv2 next to this file.`,
    "",
    `The complete source code of this build is published with every Opsis release, as files attached to ${RELEASES}/v${release}, and can also be fetched from where it was built:`,
    "",
    ...manifest.sources.map((s) => `- ${s.name}: ${s.what}. ${s.url}`),
    "",
    `Remotion builds these libraries with the scripts in rust-ffmpeg-splitter at ${manifest.chain.splitter}, packaged by rust-ffmpeg-sys at ${manifest.chain.rustFfmpegSys} and rust-ffmpeg at ${manifest.chain.rustFfmpeg}, which its compositor names in ${manifest.chain.compositor}.`,
    "",
  ];
  return lines.join("\n");
}
