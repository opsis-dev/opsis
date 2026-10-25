import path from "node:path";
import {
  googleFontModuleFile,
  GOOGLE_FONT_IMPORT,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import type { WebpackOverrideFn } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";

export const makeWebpackOverride =
  (videoDir: string, aliases: Record<string, string> = {}): WebpackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    const fontShims = path.join(videoDir, ...GOOGLE_SHIM_SEGMENTS);
    const alias = withTailwind.resolve?.alias;
    return {
      ...withTailwind,
      plugins: [
        ...(withTailwind.plugins ?? []),
        {
          apply(compiler) {
            compiler.hooks.normalModuleFactory.tap("opsis-local-fonts", (factory) => {
              factory.hooks.beforeResolve.tap("opsis-local-fonts", (data) => {
                const name = GOOGLE_FONT_IMPORT.exec(data.request)?.[1];
                if (name) data.request = path.join(fontShims, googleFontModuleFile(name));
              });
            });
          },
        },
      ],
      resolve: {
        ...withTailwind.resolve,
        alias: {
          ...(Array.isArray(alias) ? {} : alias),
          ...aliases,
        },
      },
    };
  };
