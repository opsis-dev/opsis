import { publicFile } from "@opsis/runtime/assets.ts";
import { Preloaded, cell } from "#oracle/parts.tsx";

const PHOTO = publicFile("placeholders/launch-earth.jpg");

const FILTERS = [
  "none",
  "brightness(1.6)",
  "brightness(0.4)",
  "saturate(0)",
  "saturate(2.5)",
  "contrast(1.8)",
  "sepia(1)",
  "hue-rotate(120deg) contrast(0.8) brightness(1.1)",
];

export function Case() {
  return (
    <Preloaded urls={[PHOTO]}>
      {FILTERS.map((filter, i) => (
        <div
          key={i}
          style={cell(i, {
            filter,
            background: `linear-gradient(transparent 60%, #f97316 60% 80%, #0ea5e9 80%), url("${PHOTO}") center / cover`,
          })}
        />
      ))}
    </Preloaded>
  );
}
