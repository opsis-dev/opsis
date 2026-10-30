import { render } from "@testing-library/react";
import { useCurrentFrame } from "remotion";
import { expect, it } from "vitest";
import { shutterOffsets } from "#video/camera/lib/shutter.ts";
import { MotionBlur } from "#video/camera/motion-blur.tsx";
import { TestFrame } from "#video/core/test-frame.tsx";

function FrameShown({ muted }: { muted: boolean }) {
  const frame = useCurrentFrame();
  return <span data-frame={frame} data-muted={muted} />;
}

function Blurred() {
  return (
    <MotionBlur shutterAngle={180} samples={6}>
      {(muted) => <FrameShown muted={muted} />}
    </MotionBlur>
  );
}

it("shows each sample at its own time inside the open shutter, with only the newest heard", () => {
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
  expect(shown.map((span) => span.dataset.muted)).toEqual([
    "false",
    ...Array.from({ length: 5 }, () => "true"),
  ]);
});
