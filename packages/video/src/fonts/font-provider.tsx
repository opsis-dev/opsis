import { useHeldLoad } from "@opsis/runtime/holds.ts";
import React from "react";
import { applyFilmFonts, NO_FONTS } from "#video/fonts/registry.ts";
import type { FilmFonts } from "#video/generated/spec.ts";

export function FontProvider({
  fonts,
  children,
}: {
  fonts?: FilmFonts;
  children: React.ReactNode;
}) {
  const film = fonts ?? NO_FONTS;
  const loaded = useHeldLoad("Loading fonts", JSON.stringify(film), () => applyFilmFonts(film));
  return loaded === null ? null : <>{children}</>;
}
