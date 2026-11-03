import { FootageFrame } from "#video/core/footage.tsx";
import { TerminalScreen } from "#video/core/terminal-screen.tsx";
import type { Fit, ScreenProps } from "#video/generated/spec.ts";

export function Screen({ screen, timeS, fit }: { screen: ScreenProps; timeS: number; fit: Fit }) {
  switch (screen.kind) {
    case "footage":
      return screen.footageUrl ? (
        <FootageFrame url={screen.footageUrl} timeS={timeS} fit={fit} />
      ) : null;
    case "terminal":
      return <TerminalScreen screen={screen} timeS={timeS} fit={fit} />;
  }
}

export function hasScreen(screen: ScreenProps): boolean {
  switch (screen.kind) {
    case "footage":
      return Boolean(screen.footageUrl);
    case "terminal":
      return Boolean(screen.framesUrl);
  }
}
