import type { RspackOverrideFn } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";

export function makeBundlerOverride(aliases: Record<string, string> = {}): RspackOverrideFn {
  return (config) => {
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
}
