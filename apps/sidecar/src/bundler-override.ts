import path from "node:path";
import {
  googleFontModuleFile,
  GOOGLE_FONT_IMPORT,
  GOOGLE_SHIM_SEGMENTS,
} from "@opsis/video/fonts/google-font-imports";
import type { RspackOverrideFn } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";
import type { RspackPluginInstance } from "@rspack/core";

const localFonts = (fontShims: string): RspackPluginInstance => ({
  apply(compiler) {
    compiler.hooks.normalModuleFactory.tap("opsis-local-fonts", (factory) => {
      factory.hooks.beforeResolve.tap("opsis-local-fonts", (data) => {
        const name = GOOGLE_FONT_IMPORT.exec(data.request)?.[1];
        if (name) data.request = path.join(fontShims, googleFontModuleFile(name));
      });
    });
  },
});

export const makeBundlerOverride =
  (videoDir: string, aliases: Record<string, string> = {}): RspackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    const alias = withTailwind.resolve?.alias;
    return {
      ...withTailwind,
      plugins: [
        ...(withTailwind.plugins ?? []),
        localFonts(path.join(videoDir, ...GOOGLE_SHIM_SEGMENTS)),
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
