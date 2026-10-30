import type { RspackOverrideFn } from "@remotion/bundler";

export function makeBundlerOverride(aliases: Record<string, string> = {}): RspackOverrideFn {
  return (config) => {
    const alias = config.resolve?.alias;
    return {
      ...config,
      resolve: {
        ...config.resolve,
        alias: {
          ...(Array.isArray(alias) ? {} : alias),
          ...aliases,
        },
      },
    };
  };
}
