export const GOOGLE_FONT_IMPORT = /^@remotion\/google-fonts\/([A-Za-z]+)$/;

export const GOOGLE_SHIM_SEGMENTS = ["fonts", "google"] as const;

export const googleFontModuleFile = (name: string): string =>
  `${name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase()}.ts`;

export const googleImportName = (family: string): string => family.replaceAll(" ", "");
