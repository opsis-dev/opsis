import { TestFrame } from "@opsis/runtime/test-frame.tsx";
import { render, waitFor } from "@testing-library/react";
import { beforeAll, expect, it } from "vitest";
import type { LottieElement as Drawn } from "#video/elements/overlay/lottie.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";

let LottieElement: typeof Drawn;

beforeAll(async () => {
  Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
    value: () => ({ fillStyle: "", fillRect: () => undefined }),
  });
  ({ LottieElement } = await import("#video/elements/overlay/lottie.tsx"));
});

const SLIDING_SQUARE = {
  v: "5.7.0",
  fr: 30,
  ip: 0,
  op: 60,
  w: 100,
  h: 100,
  assets: [],
  layers: [
    {
      ty: 4,
      nm: "square",
      ind: 1,
      ip: 0,
      op: 60,
      st: 0,
      ks: {
        o: { a: 0, k: 100 },
        r: { a: 0, k: 0 },
        a: { a: 0, k: [0, 0, 0] },
        s: { a: 0, k: [100, 100, 100] },
        p: {
          a: 1,
          k: [
            { t: 0, s: [0, 50, 0], o: { x: [0], y: [0] }, i: { x: [1], y: [1] } },
            { t: 59, s: [59, 50, 0] },
          ],
        },
      },
      shapes: [
        { ty: "rc", d: 1, s: { a: 0, k: [10, 10] }, p: { a: 0, k: [0, 0] }, r: { a: 0, k: 0 } },
        { ty: "fl", c: { a: 0, k: [1, 0, 0, 1] }, o: { a: 0, k: 100 } },
      ],
    },
  ],
};

const SETTINGS: ElementSettings["lottie"] = {
  src: `data:application/json,${encodeURIComponent(JSON.stringify(SLIDING_SQUARE))}`,
  loop: true,
  speed: 1,
  backwards: false,
  fit: "contain",
  background: "transparent",
  align: "center",
};

async function squareAt(frame: number, changes: Partial<ElementSettings["lottie"]> = {}) {
  function Animation() {
    return <LottieElement {...SETTINGS} {...changes} />;
  }
  const view = render(
    <TestFrame
      component={Animation}
      durationInFrames={300}
      fps={30}
      compositionWidth={100}
      compositionHeight={100}
      frameToDisplay={frame}
    />,
  );
  const moved = await waitFor(() => {
    const layer = view.container.querySelector("g[transform^='matrix']:not([clip-path])");
    const transform = layer?.getAttribute("transform");
    if (!transform) throw new Error("the animation hasn't drawn yet");
    return transform;
  });
  view.unmount();
  return Number(moved.split(",")[4]);
}

it("shows the animation frame the film's time asks for", async () => {
  expect(await squareAt(15)).toBeCloseTo(15, 3);
  expect(await squareAt(15, { speed: 2 })).toBeCloseTo(30, 3);
  expect(await squareAt(15, { backwards: true })).toBeCloseTo(44, 3);
  expect(await squareAt(75)).toBeCloseTo(15, 3);
  expect(await squareAt(75, { loop: false })).toBeCloseTo(59, 3);
});
