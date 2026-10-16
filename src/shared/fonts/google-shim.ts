export const GOOGLE_FONT_IMPORT = /^@remotion\/google-fonts\/([A-Za-z]+)$/;

export const fontShimFile = (name: string): string =>
  `${name.replace(/([a-z0-9])([A-Z])/g, "$1-$2").toLowerCase()}.ts`;
