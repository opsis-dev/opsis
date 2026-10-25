import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { ROOT } from "./tool-config";

type KeyDescription = {
  key: string;
  keyCode: number;
  keyCodeWithoutLocation: number;
  code: string;
  text: string;
  location: number;
  shifted?: KeyDescription;
};

type TextMatch = string | RegExp;

export type PlaywrightInternals = {
  version: string;
  clockSource: string;
  injectedSource: string;
  keyboardLayout: Record<string, unknown>;
  keypadLocation: number;
  macEditingCommands: Record<string, string | string[]>;
  fontFamilies: Record<string, unknown>;
  switches: string[];
  keyMap: Map<string, KeyDescription>;
  globToRegexPattern: (glob: string) => string;
  urlMatches: (baseURL: string | undefined, url: string, match: string) => boolean;
  escapeForTextSelector: (text: TextMatch, exact: boolean) => string;
  getByTestIdSelector: (attribute: string, testId: TextMatch) => string;
  getByLabelSelector: (text: TextMatch, options?: { exact?: boolean }) => string;
  getByPlaceholderSelector: (text: TextMatch, options?: { exact?: boolean }) => string;
  getByTextSelector: (text: TextMatch, options?: { exact?: boolean }) => string;
  getByRoleSelector: (role: string, options?: { name?: TextMatch; exact?: boolean }) => string;
};

const INITS = [
  "init_clockSource",
  "init_injectedScriptSource",
  "init_usKeyboardLayout",
  "init_macEditingCommands",
  "init_defaultFontFamilies",
  "init_chromiumSwitches",
  "init_input",
  "init_urlMatch",
  "init_stringUtils",
  "init_locatorUtils",
];

function sourceVariable(bundle: string, init: string): string {
  const found = new RegExp(`var ${init} = __esm\\(\\{\\n[^\\n]*\\n[^\\n]*\\n\\s*(\\w+) = `).exec(
    bundle,
  );
  if (!found) throw new Error(`${init} isn't in this playwright-core`);
  return found[1];
}

let loaded: PlaywrightInternals | null = null;

export function playwrightInternals(): PlaywrightInternals {
  if (loaded) return loaded;
  const require = createRequire(path.join(ROOT, "package.json"));
  const dir = path.dirname(require.resolve("playwright-core/package.json"));
  const { version } = JSON.parse(readFileSync(path.join(dir, "package.json"), "utf8")) as {
    version: string;
  };
  const file = path.join(dir, "lib", "coreBundle.js");
  const bundle = readFileSync(file, "utf8");
  const read = new Function(
    "module",
    "exports",
    "require",
    "__filename",
    "__dirname",
    `${bundle}
${INITS.map((init) => `${init}();`).join("\n")}
return {
  clockSource: ${sourceVariable(bundle, "init_clockSource")},
  injectedSource: ${sourceVariable(bundle, "init_injectedScriptSource")},
  keyboardLayout: USKeyboardLayout,
  keypadLocation,
  macEditingCommands,
  fontFamilies: platformToFontFamilies,
  switches: chromiumSwitches(),
  keyMap: usKeyboardLayout,
  globToRegexPattern,
  urlMatches,
  escapeForTextSelector,
  getByTestIdSelector,
  getByLabelSelector,
  getByPlaceholderSelector,
  getByTextSelector,
  getByRoleSelector,
};`,
  ) as (...args: unknown[]) => Omit<PlaywrightInternals, "version">;
  const module = { exports: {} };
  loaded = {
    version,
    ...read(module, module.exports, createRequire(file), file, path.dirname(file)),
  };
  return loaded;
}
