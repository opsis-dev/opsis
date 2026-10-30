import { render } from "@testing-library/react";
import { expect, it } from "vitest";
import { TestFrame } from "#video/core/test-frame.tsx";
import { LogoAssemble } from "#video/elements/scene/logo-assemble.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import DEFAULT_PROPS from "#video/registry/default-props.generated.json";

const usual = DEFAULT_PROPS["logo-assemble"] as unknown as ElementSettings["logo-assemble"];

it("never loads sample pictures from the internet when no images are set", () => {
  function Cleared() {
    return <LogoAssemble {...usual} images={[]} logoSrc="" />;
  }
  const sources = [30, 90, 150].flatMap((frame) => {
    const view = render(
      <TestFrame
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
