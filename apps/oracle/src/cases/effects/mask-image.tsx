import { publicFile } from "@opsis/runtime/assets.ts";
import type { CSSProperties } from "react";
import { Preloaded, cell } from "#oracle/parts.tsx";

const LOGO = publicFile("placeholders/logo-mark.svg");

const MASKS: CSSProperties[] = [
  { maskImage: "linear-gradient(#000, transparent)" },
  { maskImage: "linear-gradient(90deg, transparent, #000 30% 70%, transparent)" },
  { maskImage: "radial-gradient(circle, #000 40%, transparent 70%)" },
  { maskImage: "radial-gradient(circle, #000 3px, transparent 3.5px)", maskSize: "12px 12px" },
  {
    maskImage: "linear-gradient(#000, #000)",
    maskSize: "50% 50%",
    maskRepeat: "no-repeat",
    maskPosition: "right bottom",
  },
  { maskImage: "repeating-linear-gradient(90deg, #000 0 6px, transparent 6px 12px)" },
  {
    maskImage: `url("${LOGO}")`,
    maskSize: "contain",
    maskRepeat: "no-repeat",
    maskPosition: "center",
  },
  { WebkitMaskImage: "linear-gradient(135deg, #000 50%, rgba(0,0,0,0.2))" },
];

export function Case() {
  return (
    <Preloaded urls={[LOGO]}>
      {MASKS.map((style, i) => (
        <div
          key={i}
          style={cell(i, {
            background: "linear-gradient(135deg, #f97316, #7c3aed 60%, #0ea5e9)",
            ...style,
          })}
        />
      ))}
    </Preloaded>
  );
}
