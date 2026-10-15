import { continueRender, delayRender, staticFile } from "remotion";

export const INTER_FAMILY = "Inter";

let started = false;

export function loadFonts(): void {
  if (started) return;
  started = true;
  if (typeof document === "undefined" || typeof FontFace === "undefined") return;

  const handle = delayRender(`loading ${INTER_FAMILY}`);
  const face = new FontFace(
    INTER_FAMILY,
    `url(${staticFile("fonts/inter-latin-wght-normal.woff2")}) format("woff2")`,
    { weight: "100 900", style: "normal", display: "block" },
  );

  face
    .load()
    .then((loaded) => {
      document.fonts.add(loaded);
      continueRender(handle);
    })
    .catch(() => {
      continueRender(handle);
    });
}
