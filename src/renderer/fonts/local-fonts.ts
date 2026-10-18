export async function installedFonts(): Promise<LocalFontData[]> {
  if (!window.queryLocalFonts)
    throw new Error("This computer doesn't let apps list installed fonts.");
  return window.queryLocalFonts();
}
