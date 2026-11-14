import type { KnipConfig } from "knip";

const FILE_READ_PACKAGES = [
  "@hugeicons/core-free-icons",
  "@lucide/lab",
  "@mynaui/icons",
  "@tabler/icons",
  "iconoir",
];

export default {
  ignoreIssues: {
    "scripts/lib/common/*.ts": ["exports"],
    "scripts/lib/generated/*.ts": ["types"],
  },
  ignoreBinaries: [
    "zig",
    "rustc",
    "dbus-daemon",
    "gnome-keyring-daemon",
    "sway",
    "Xvfb",
    "fc-match",
    "uvx",
    /^clang\+\+$/,
    "cmake",
  ],
  workspaces: {
    ".": {
      entry: ["scripts/*.ts", ".claude/skills/*/*.ts"],
      project: ["scripts/**/*.ts", ".claude/skills/**/*.ts"],
      ignoreDependencies: FILE_READ_PACKAGES,
    },
  },
} satisfies KnipConfig;
