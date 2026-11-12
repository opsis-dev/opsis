import { publicFile } from "@opsis/runtime/assets.ts";
import type { CSSProperties } from "react";
import { Preloaded, SLATE, cell } from "#oracle/parts.tsx";

const PHOTO = publicFile("placeholders/launch-earth.jpg");
const SCREEN = publicFile("placeholders/screen-tall.svg");

const FITS: [string, CSSProperties][] = [
  [PHOTO, { objectFit: "cover" }],
  [PHOTO, { objectFit: "contain" }],
  [PHOTO, { objectFit: "fill" }],
  [PHOTO, { objectFit: "none" }],
  [PHOTO, { objectFit: "cover", objectPosition: "left top" }],
  [PHOTO, { objectFit: "scale-down" }],
  [SCREEN, { objectFit: "contain" }],
  [SCREEN, { objectFit: "cover", objectPosition: "50% 0%" }],
];

export function Case() {
  return (
    <Preloaded urls={[PHOTO, SCREEN]}>
      {FITS.map(([src, style], i) => (
        <div key={i} style={cell(i, { background: SLATE })}>
          <img
            src={src}
            alt=""
            style={{ width: "100%", height: "100%", maxWidth: "none", ...style }}
          />
        </div>
      ))}
    </Preloaded>
  );
}
