import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { LogoAssemble } from "../elements/snap-cn/logo-assemble";

it("never loads sample pictures from the internet when no images are set", () => {
  const Cleared = () => <LogoAssemble brandName="Acme" />;
  const sources = [30, 90, 150].flatMap((frame) => {
    const view = render(
      <Thumbnail
        component={Cleared}
        durationInFrames={180}
        fps={30}
        compositionWidth={1280}
        compositionHeight={720}
        frameToDisplay={frame}
      />,
    );
    const found = [...view.container.querySelectorAll("img")].map((img) => img.getAttribute("src"));
    view.unmount();
    return found;
  });
  expect(sources).toEqual([]);
});
