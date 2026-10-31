import { cleanup, render } from "@testing-library/react";
import * as fc from "fast-check";
import { afterEach, describe, expect, it } from "vitest";
import { FilmClock, Freeze, Loop, Sequence, useFilm, useFrame } from "#runtime/clock.tsx";

afterEach(cleanup);

const FILM = { width: 1920, height: 1080, fps: 30, durationInFrames: 300 };

function Seen() {
  const frame = useFrame();
  const { durationInFrames, width, height } = useFilm();
  return <i data-testid="seen">{JSON.stringify({ frame, durationInFrames, width, height })}</i>;
}

function seen(view: ReturnType<typeof render>) {
  const shown = view.container.querySelector("[data-testid=seen]");
  return shown ? (JSON.parse(shown.textContent ?? "") as Record<string, number>) : null;
}

describe("Sequence", () => {
  it("shows its children only inside its span, counting frames from its start", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 299 }),
        fc.integer({ min: -50, max: 250 }),
        fc.integer({ min: 1, max: 200 }),
        (frame, from, length) => {
          const view = render(
            <FilmClock film={FILM} frame={frame}>
              <Sequence from={from} durationInFrames={length}>
                <Seen />
              </Sequence>
            </FilmClock>,
          );
          const inside = frame >= from && frame < from + length;
          expect(seen(view)).toEqual(
            inside
              ? {
                  frame: frame - from,
                  durationInFrames: Math.min(length, FILM.durationInFrames - from),
                  width: FILM.width,
                  height: FILM.height,
                }
              : null,
          );
          cleanup();
        },
      ),
    );
  });

  it("adds up nested starts and keeps a child's length inside its parent's", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 100 }),
        fc.integer({ min: 0, max: 100 }),
        fc.integer({ min: 0, max: 299 }),
        (outer, inner, frame) => {
          const view = render(
            <FilmClock film={FILM} frame={frame}>
              <Sequence from={outer} durationInFrames={150} layout="none" width={640} height={360}>
                <Sequence from={inner}>
                  <Seen />
                </Sequence>
              </Sequence>
            </FilmClock>,
          );
          const local = frame - outer - inner;
          const hidden = frame < outer || frame >= outer + 150 || local < 0;
          expect(seen(view)).toEqual(
            hidden
              ? null
              : { frame: local, durationInFrames: 150 - inner, width: 640, height: 360 },
          );
          cleanup();
        },
      ),
    );
  });
});

describe("Freeze and Loop", () => {
  it("holds a frozen frame, fractional ones too", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 299 }),
        fc.double({ min: 0, max: 100, noNaN: true }),
        (frame, held) => {
          const view = render(
            <FilmClock film={FILM} frame={frame}>
              <Freeze frame={held}>
                <Seen />
              </Freeze>
            </FilmClock>,
          );
          expect(seen(view)?.frame).toBe(held);
          cleanup();
        },
      ),
    );
  });

  it("repeats its length over the film", () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 0, max: 299 }),
        fc.integer({ min: 1, max: 120 }),
        (frame, length) => {
          const view = render(
            <FilmClock film={FILM} frame={frame}>
              <Loop durationInFrames={length}>
                <Seen />
              </Loop>
            </FilmClock>,
          );
          const start = frame - (frame % length);
          expect(seen(view)).toMatchObject({
            frame: frame % length,
            durationInFrames: Math.min(length, FILM.durationInFrames - start),
          });
          cleanup();
        },
      ),
    );
  });
});
