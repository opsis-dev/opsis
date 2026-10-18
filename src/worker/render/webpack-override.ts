import path from "node:path";
import type { WebpackOverrideFn } from "@remotion/bundler";
import { enableTailwind } from "@remotion/tailwind-v4";
import { googleFontModuleFile, GOOGLE_FONT_IMPORT } from "../../shared/fonts/google-font-imports";

export const makeWebpackOverride =
  (projectRoot: string): WebpackOverrideFn =>
  (config) => {
    const withTailwind = enableTailwind(config);
    const fontShims = path.join(projectRoot, "src", "video", "fonts", "google");
    const alias = withTailwind.resolve?.alias;
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
          ...(Array.isArray(alias) ? {} : alias),
          "@": path.join(projectRoot, "src"),
        },
      },
    };
  };
