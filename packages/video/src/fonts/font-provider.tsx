import React from "react";
import { useHeldLoad } from "#video/core/loading.ts";
import { applyFilmFonts, NO_FONTS } from "#video/fonts/registry.ts";
import type { FilmFonts } from "#video/generated/spec.ts";

export const FontProvider: React.FC<{ fonts?: FilmFonts; children: React.ReactNode }> = ({
  fonts,
  children,
}) => {
  const film = fonts ?? NO_FONTS;
  const loaded = useHeldLoad("Loading fonts", JSON.stringify(film), () => applyFilmFonts(film));
  return loaded === null ? null : <>{children}</>;
};
