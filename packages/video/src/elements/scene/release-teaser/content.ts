import type { ElementSettings } from "#video/generated/spec.ts";

export function releaseTeaserScene(settings: ElementSettings["release-teaser"]) {
  return {
    brandName: settings.brandName,
    release: settings.release.trim(),
    tagline: settings.tagline,
    statements: settings.statements,
    theme: {
      background: settings.themeBackground,
      surface: settings.themeSurface,
      foreground: settings.themeForeground,
      muted: settings.themeMuted,
      accent: settings.accentColor,
    },
    logoSrc: settings.logoSrc?.trim() || undefined,
    lightIntensity: settings.lightIntensity,
    reducedMotion: settings.reducedMotion,
    textSize: settings.textSize,
    statementWeight: settings.statementWeight,
    logoSize: settings.logoSize,
  };
}

export type ReleaseTeaserScene = ReturnType<typeof releaseTeaserScene>;
