import type { KnipConfig } from "knip";

const FILE_READ_PACKAGES = [
  /^@fontsource(-variable)?\//,
  "@hugeicons/core-free-icons",
  "@lucide/lab",
  "@mynaui/icons",
  "@tabler/icons",
  "iconoir",
  "source-map-explorer",
];

function cssImports(text: string): string {
  return [...text.matchAll(/(?<=@)import[^;]+/g)].join("\n");
}

export default {
  ignore: ["**/*.generated.*"],
  ignoreBinaries: [
    "zig",
    "rustc",
    "dbus-daemon",
    "gnome-keyring-daemon",
    "sway",
    "Xvfb",
    "fc-match",
    "uvx",
  ],
  compilers: { css: cssImports },
  workspaces: {
    ".": {
      entry: ["scripts/*.ts", ".claude/skills/*/*.ts"],
      project: ["scripts/**/*.ts", ".claude/skills/**/*.ts"],
      ignoreDependencies: FILE_READ_PACKAGES,
    },
    "apps/film": {
      project: ["src/**/*.{ts,tsx,css}"],
    },
    "apps/terminal": {
      project: ["src/**/*.{ts,tsx,css}"],
    },
    "packages/video": {
      project: ["src/**/*.{ts,tsx,css}"],
    },
    "packages/runtime": {
      project: ["src/**/*.{ts,tsx,css}"],
    },
  },
} satisfies KnipConfig;
