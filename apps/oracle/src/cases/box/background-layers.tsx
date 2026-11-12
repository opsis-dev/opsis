import { publicFile } from "@opsis/runtime/assets.ts";
import type { CSSProperties } from "react";
import { Preloaded, cell } from "#oracle/parts.tsx";

const PHOTO_FILE = publicFile("placeholders/launch-earth.jpg");
const PHOTO = `url("${PHOTO_FILE}")`;
const DOTS = "radial-gradient(circle, #f8fafc 2px, transparent 2.5px)";

const LAYERS: CSSProperties[] = [
  { backgroundImage: DOTS, backgroundSize: "16px 16px" },
  {
    backgroundImage: DOTS,
    backgroundSize: "20px 20px",
    backgroundPosition: "10px 10px",
    backgroundColor: "#7c3aed",
  },
  {
    backgroundImage: "linear-gradient(#f97316, #0ea5e9)",
    backgroundSize: "50% 50%",
    backgroundRepeat: "no-repeat",
  },
  {
    backgroundImage: "linear-gradient(#22c55e, #0f172a)",
    backgroundSize: "40px 40px",
    backgroundRepeat: "repeat-x",
  },
  { backgroundImage: PHOTO, backgroundSize: "cover", backgroundPosition: "center" },
  {
    backgroundImage: PHOTO,
    backgroundSize: "contain",
    backgroundRepeat: "no-repeat",
    backgroundColor: "#334155",
  },
  {
    background: `linear-gradient(rgba(15,23,42,0.2), rgba(15,23,42,0.9)), ${PHOTO} 30% 50% / 200% auto no-repeat`,
  },
  {
    backgroundImage: `${DOTS}, linear-gradient(90deg, #f43f5e, #facc15)`,
    backgroundSize: "12px 12px, 100% 100%",
    backgroundPosition: "right bottom, 0 0",
  },
];

export function Case() {
  return (
    <Preloaded urls={[PHOTO_FILE]}>
      {LAYERS.map((style, i) => (
        <div key={i} style={cell(i, style)} />
      ))}
    </Preloaded>
  );
}
