import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { TestFrame } from "#video/core/test-frame.tsx";
import { SpeedLines } from "#video/elements/effect/speed-lines.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

const DEFAULTS: ElementSettings["speed-lines"] = {
  mode: "trail",
  count: 12,
  weight: 2,
  spread: 96,
  duration: 14,
  bounce: 0.5,
  x: 0.5,
  y: 0.5,
  delay: 0,
  color: "#ffffff",
  seed: 1,
  speed: 1,
  fromX: -0.25,
  fromY: 0.5,
  hold: 8,
  streakLength: 1.6,
};

function drawn(mode: "trail" | "focus", frame: number) {
  function Lines() {
    return (
      <SpeedLines {...DEFAULTS} mode={mode}>
        {null}
      </SpeedLines>
    );
  }
  const view = render(
    <TestFrame
      component={Lines}
      durationInFrames={60}
      fps={30}
      compositionWidth={320}
      compositionHeight={180}
      frameToDisplay={frame}
    />,
  );
  return {
    path: view.container.querySelector("path")?.getAttribute("d") ?? "",
    discs: view.container.querySelectorAll("ellipse").length,
  };
}

it("draws only its lines", () => {
  const trail = drawn("trail", 6);
  expect(trail.path).not.toBe("");
  expect(trail.discs).toBe(0);
  expect(drawn("focus", 4).path).not.toBe("");
});
