import path from "node:path";
import { enableTailwind } from "@remotion/tailwind-v4";
import type { WebpackOverrideFn } from "@remotion/bundler";
import { googleFontModuleFile, GOOGLE_FONT_IMPORT } from "../../shared/fonts/google-font-imports";

export const makeWebpackOverride =
  (projectRoot: string): WebpackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    const fontShims = path.join(projectRoot, "src", "video", "fonts", "google");
    return {
      ...withTailwind,
      plugins: [
        ...(withTailwind.plugins ?? []),
        {
          apply(compiler) {
            compiler.hooks.normalModuleFactory.tap("demoflow-local-fonts", (factory) => {
              factory.hooks.beforeResolve.tap("demoflow-local-fonts", (data) => {
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
          ...(withTailwind.resolve?.alias as Record<string, string> | undefined),
          "@": path.join(projectRoot, "src"),
        },
      },
    };
  };
