import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { SpeedLines } from "#video/elements/effect/speed-lines.tsx";

const drawn = (mode: "trail" | "focus", frame: number) => {
  const Lines = () => <SpeedLines mode={mode}>{null}</SpeedLines>;
  const view = render(
    <Thumbnail
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
};

it("draws only its lines", () => {
  const trail = drawn("trail", 6);
  expect(trail.path).not.toBe("");
  expect(trail.discs).toBe(0);
  expect(drawn("focus", 4).path).not.toBe("");
});
