import type { FilmDrawing } from "@opsis/runtime/page.tsx";
import { FontProvider } from "@opsis/video/fonts/font-provider";
import type { TimelineSpec } from "@opsis/video/generated/spec";
import "@opsis/video/video.css";
import { caseNamed } from "#oracle/cases.ts";
import { STAGE } from "#oracle/parts.tsx";

function requestedCase(): string {
  return new URLSearchParams(window.location.search).get("case") ?? "";
}

export const CASE_FILM: FilmDrawing<TimelineSpec> = {
  film(spec) {
    return { width: spec.width, height: spec.height, fps: spec.fps, durationInFrames: 1 };
  },
  draw(spec) {
    const Drawn = caseNamed(requestedCase());
    return (
      <FontProvider fonts={spec.fonts}>
        <div style={STAGE}>
          <Drawn />
        </div>
      </FontProvider>
    );
  },
};
