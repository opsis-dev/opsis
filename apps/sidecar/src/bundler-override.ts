import type { RspackOverrideFn } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";

export const makeBundlerOverride =
  (aliases: Record<string, string> = {}): RspackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    const alias = withTailwind.resolve?.alias;
    return {
      ...withTailwind,
      resolve: {
        ...withTailwind.resolve,
        alias: {
          ...(Array.isArray(alias) ? {} : alias),
          ...aliases,
        },
      },
    };
  };
