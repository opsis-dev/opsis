export const VIDEO_SOURCE = ["packages", "video", "src"] as const;

export type ExportedSource = { from: readonly string[]; to: string };

export const EXPORTED_SOURCES: readonly ExportedSource[] = [
  { from: VIDEO_SOURCE, to: "video" },
  {
    from: ["apps", "sidecar", "src", "bundler-override.ts"],
    to: "sidecar/bundler-override.ts",
  },
];

export const EXPORTED_PACKAGES = { "@opsis/video": "video" } as const;

const manifest = (folder: "apps" | "packages", name: string): ExportedSource => ({
  from: [folder, name, "package.json"],
  to: `manifests/${name}.json`,
});

export const packageManifest = (name: string): ExportedSource => manifest("packages", name);

export const DEPENDENCY_MANIFESTS: readonly ExportedSource[] = [
  { from: ["package.json"], to: "manifests/app.json" },
  packageManifest("video"),
  manifest("apps", "ui"),
  manifest("apps", "sidecar"),
];
