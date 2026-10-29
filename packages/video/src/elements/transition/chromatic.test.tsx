import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { useEffect } from "react";
import { expect, it } from "vitest";
import { chromaticAberrationWipe } from "#video/elements/transition/chromatic-aberration-wipe.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

const SETTINGS: ElementSettings["chromatic-aberration-wipe"] = {
  intensity: 12,
  axis: "horizontal",
  slide: 1,
  fade: false,
};

it("keeps the scene mounted as the colour split opens", () => {
  const events: string[] = [];
  const Scene = () => {
    useEffect(() => {
      events.push("mount");
      return () => {
        events.push("unmount");
      };
    }, []);
    return <p>scene</p>;
  };
  const { component: Presentation } = chromaticAberrationWipe(SETTINGS);
  const AtProgress = ({ progress }: { progress: number }) => (
    <Presentation
      presentationProgress={progress}
      presentationDirection="entering"
      presentationDurationInFrames={20}
      passedProps={SETTINGS}
      onElementImage={() => undefined}
      onUnmount={() => undefined}
      bothEnteringAndExiting={false}
    >
      <Scene />
    </Presentation>
  );
  const thumbnail = (progress: number) => (
    <Thumbnail
      component={AtProgress}
      inputProps={{ progress }}
      durationInFrames={20}
      fps={30}
      compositionWidth={320}
      compositionHeight={180}
      frameToDisplay={0}
    />
  );
  const view = render(thumbnail(0));
  events.length = 0;
  view.rerender(thumbnail(0.5));
  expect(events.filter((event) => event === "unmount")).toEqual([]);
});
