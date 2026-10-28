import { Thumbnail } from "@remotion/player";
import { render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SpringSettleItem, SpringSettleScenes } from "#video/elements/scene/spring-settle/index.tsx";

afterEach(() => {
  vi.restoreAllMocks();
});

const errorsAtFrame = (Component: React.FC, frame: number) => {
  const errors: string[] = [];
  vi.spyOn(console, "error").mockImplementation((...args: unknown[]) => {
    errors.push(args.map(String).join(" "));
  });
  render(
    <Thumbnail
      component={Component}
      durationInFrames={120}
      fps={30}
      compositionWidth={320}
      compositionHeight={180}
      frameToDisplay={frame}
    />,
  );
  return errors;
};

it("draws nothing for no scenes instead of failing", () => {
  const Empty = () => <SpringSettleScenes scenes={[]} />;
  expect(errorsAtFrame(Empty, 10)).toEqual([]);
});

it("shows the first scene with no fade-in instead of failing", () => {
  const Instant = () => (
    <SpringSettleScenes
      scenes={[
        {
          content: (
            <SpringSettleItem>
              <p>first</p>
            </SpringSettleItem>
          ),
          durationInFrames: 60,
        },
      ]}
      config={{ enterFadeFrames: 0 }}
    />
  );
  expect(errorsAtFrame(Instant, 0)).toEqual([]);
});
