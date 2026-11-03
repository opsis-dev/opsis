import { FootageFrame } from "#video/core/footage.tsx";
import type { Fit, ScreenProps } from "#video/generated/spec.ts";

export function Screen({ screen, timeS, fit }: { screen: ScreenProps; timeS: number; fit: Fit }) {
  switch (screen.kind) {
    case "footage":
      return screen.footageUrl ? (
        <FootageFrame url={screen.footageUrl} timeS={timeS} fit={fit} />
      ) : null;
  }
}

export function hasScreen(screen: ScreenProps): boolean {
  switch (screen.kind) {
    case "footage":
      return Boolean(screen.footageUrl);
  }
}
