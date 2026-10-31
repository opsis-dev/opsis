import { TestFrame } from "@opsis/runtime/test-frame.tsx";
import { render } from "@testing-library/react";
import { useCurrentFrame } from "remotion";
import { expect, it } from "vitest";
import { shutterOffsets } from "#video/camera/lib/shutter.ts";
import { MotionBlur } from "#video/camera/motion-blur.tsx";

function FrameShown() {
  const frame = useCurrentFrame();
  return <span data-frame={frame} />;
}

function Blurred() {
  return (
    <MotionBlur shutterAngle={180} samples={6}>
      <FrameShown />
    </MotionBlur>
  );
}

it("shows each sample at its own time inside the open shutter", () => {
  const view = render(
    <TestFrame
      component={Blurred}
      durationInFrames={60}
      fps={30}
      compositionWidth={100}
      compositionHeight={100}
      frameToDisplay={20}
    />,
  );
  const shown = [...view.container.querySelectorAll("span")];
  expect(shown.map((span) => Number(span.dataset.frame))).toEqual(
    shutterOffsets(180, 6).map((offset) => 20 - offset),
  );
});
