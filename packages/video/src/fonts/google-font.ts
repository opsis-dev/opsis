import { googleImportName } from "./google-font-imports";
import { ensureFontNow } from "./registry";

type LoadFontResult = {
  fontFamily: string;
  fonts: Record<string, Record<string, Record<string, string>>>;
  unicodeRanges: Record<string, string>;
  waitUntilDone: () => Promise<undefined>;
};

export function googleFont(family: string) {
  const fontFamily = `"${family}"`;
  return {
    loadFont: (..._args: unknown[]): LoadFontResult => {
      ensureFontNow(family);
      return {
        fontFamily,
        fonts: {},
        unicodeRanges: {},
        waitUntilDone: () => Promise.resolve(undefined),
      };
    },
    getInfo: () => ({ fontFamily, importName: googleImportName(family) }),
  };
}
