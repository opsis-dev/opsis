import path from "node:path";
import { enableTailwind } from "@remotion/tailwind-v4";
import type { WebpackOverrideFn } from "@remotion/bundler";

const GOOGLE_FONT = /^@remotion\/google-fonts\/([A-Za-z]+)$/;
const kebab = (name: string) => name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase();

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
                const name = GOOGLE_FONT.exec(data.request)?.[1];
                if (name) data.request = path.join(fontShims, `${kebab(name)}.ts`);
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
