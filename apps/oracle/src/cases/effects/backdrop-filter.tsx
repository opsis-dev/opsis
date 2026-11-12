import { publicFile } from "@opsis/runtime/assets.ts";
import { Preloaded, cell } from "#oracle/parts.tsx";

const PHOTO = publicFile("placeholders/launch-earth.jpg");

const FILTERS = [
  "blur(4px)",
  "blur(16px)",
  "blur(12px) saturate(1.6)",
  "brightness(0.5)",
  "blur(8px) brightness(1.3)",
  "contrast(1.5) blur(2px)",
  "blur(24px)",
  "none",
];

export function Case() {
  return (
    <Preloaded urls={[PHOTO]}>
      {FILTERS.map((backdropFilter, i) => (
        <div
          key={i}
          style={cell(i, {
            background: `repeating-linear-gradient(45deg, #f97316 0 8px, #0f172a 8px 16px), url("${PHOTO}")`,
            backgroundBlendMode: "overlay",
            overflow: "hidden",
          })}
        >
          <div
            style={{
              position: "absolute",
              left: 16,
              top: 30,
              right: 16,
              bottom: 30,
              borderRadius: 16,
              background: "rgba(255,255,255,0.12)",
              border: "1px solid rgba(255,255,255,0.3)",
              backdropFilter,
            }}
          />
        </div>
      ))}
    </Preloaded>
  );
}
