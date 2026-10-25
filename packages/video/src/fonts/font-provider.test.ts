import { describe, expect, it } from "vitest";
import { usedFontNames } from "#video/fonts/font-provider.tsx";

describe("fonts used by a timeline", () => {
  it("collects per-node fonts from items, groups and wrappers", () => {
    expect(
      usedFontNames([
        {
          id: "t",
          items: [
            {
              id: "a",
              from: 0,
              durationInFrames: 10,
              kind: "element",
              component: "core/title",
              props: { fontFamily: '"Fraunces", serif' },
            },
            {
              id: "g",
              from: 0,
              durationInFrames: 10,
              kind: "group",
              props: {
                wrapper: {
                  component: "snapcn/laptop-frame",
                  props: { fontFamily: '"Outfit", sans-serif' },
                },
                tracks: [
                  {
                    id: "i",
                    items: [
                      {
                        id: "b",
                        from: 0,
                        durationInFrames: 5,
                        kind: "element",
                        component: "x",
                        props: { fontFamily: "Manrope" },
                      },
                    ],
                  },
                ],
              },
            },
            {
              id: "c",
              from: 0,
              durationInFrames: 10,
              kind: "element",
              component: "core/title",
              props: {},
            },
          ],
        },
      ]),
    ).toEqual(["Fraunces", "Manrope", "Outfit"]);
  });
});
