import type { ReactNode } from "react";
import { flushSync } from "react-dom";
import { createRoot } from "react-dom/client";
import { type Film, FilmClock } from "#runtime/clock.tsx";
import { markRendering } from "#runtime/environment.ts";
import { failRender, openHolds, settled } from "#runtime/holds.ts";

export type FilmPage = {
  load(spec: unknown): Promise<Film>;
  seek(frame: number): Promise<void>;
  holds(): string[];
};

declare global {
  interface Window {
    opsis?: FilmPage;
  }
}

export type FilmDrawing<S> = {
  film(spec: S): Film;
  draw(spec: S): ReactNode;
};

async function ready() {
  await settled();
  await document.fonts.ready;
  await settled();
}

export function mountFilm<S>(container: HTMLElement, { film, draw }: FilmDrawing<S>): FilmPage {
  markRendering();
  const root = createRoot(container, {
    onUncaughtError: (error) => failRender(error),
    onRecoverableError: (error) => failRender(error),
  });
  let shown: { spec: S; film: Film } | null = null;
  function show(frame: number) {
    const current = shown;
    if (!current) throw new Error("The film was asked for a frame before it was loaded.");
    flushSync(() =>
      root.render(
        <FilmClock film={current.film} frame={frame}>
          {draw(current.spec)}
        </FilmClock>,
      ),
    );
  }
  const page: FilmPage = {
    async load(spec) {
      const typed = spec as S;
      shown = { spec: typed, film: film(typed) };
      show(0);
      await ready();
      return shown.film;
    },
    async seek(frame) {
      show(frame);
      await ready();
    },
    holds: openHolds,
  };
  window.opsis = page;
  return page;
}
