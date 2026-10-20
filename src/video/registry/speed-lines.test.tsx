import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { SpeedLines } from "../elements/remocn/speed-lines";

it("gives the clip it carries the whole frame to lay itself out in", () => {
  const Carried = () => (
    <SpeedLines>
      <p data-testid="clip">Fast</p>
    </SpeedLines>
  );
  const view = render(
    <Thumbnail
      component={Carried}
      durationInFrames={60}
      fps={30}
      compositionWidth={320}
      compositionHeight={180}
      frameToDisplay={40}
    />,
  );
  const box = view.getByTestId("clip").parentElement;
  expect(box?.style.width).toBe("320px");
  expect(box?.style.height).toBe("180px");
});
