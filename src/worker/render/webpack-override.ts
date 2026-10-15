import path from "node:path";
import { enableTailwind } from "@remotion/tailwind-v4";
import type { WebpackOverrideFn } from "@remotion/bundler";

export const makeWebpackOverride =
  (projectRoot: string): WebpackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    return {
      ...withTailwind,
      resolve: {
        ...withTailwind.resolve,
        alias: {
          ...(withTailwind.resolve?.alias as Record<string, string> | undefined),
          "@": path.join(projectRoot, "src"),
        },
      },
    };
  };
