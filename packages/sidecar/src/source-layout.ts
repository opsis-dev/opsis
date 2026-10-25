export const VIDEO_SOURCE = ["packages", "video", "src"] as const;

export type ExportedSource = { from: readonly string[]; to: string };

export const EXPORTED_SOURCES: readonly ExportedSource[] = [
  { from: VIDEO_SOURCE, to: "video" },
  { from: ["packages", "core", "src"], to: "core" },
  {
    from: ["packages", "sidecar", "src", "webpack-override.ts"],
    to: "sidecar/webpack-override.ts",
  },
];

export const EXPORTED_PACKAGES = { "@opsis/core": "core", "@opsis/video": "video" } as const;

export const DEPENDENCY_MANIFESTS: readonly ExportedSource[] = [
  { from: ["package.json"], to: "manifests/app.json" },
  ...["core", "video", "ui", "sidecar"].map((name) => ({
    from: ["packages", name, "package.json"],
    to: `manifests/${name}.json`,
  })),
];
