import { TestFrame } from "@opsis/runtime/test-frame.tsx";
import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";

function Clock() {
  const frame = useAuthoredFrame();
  const { fps, durationInFrames } = useAuthoredConfig();
  return <p>{`${frame} ${fps} ${durationInFrames}`}</p>;
}

it("counts time in frames of 30 a second at any frame rate", () => {
  const view = render(
    <TestFrame
      component={Clock}
      durationInFrames={240}
      fps={60}
      compositionWidth={320}
      compositionHeight={180}
      frameToDisplay={90}
    />,
  );
  expect(view.getByText("45 30 120")).toBeTruthy();
});
