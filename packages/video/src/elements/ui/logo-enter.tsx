import { AbsoluteFill, Img, interpolate, spring } from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";

export function LogoEnter({
  logos,
  diameter,
  overlap,
  ringColor,
  orientation,
  stagger,
  speed,
  bounce,
  ringWidth,
  shadow,
  logoScale,
  firstOnTop,
}: ElementSettings["logo-enter"]) {
  const frame = useAuthoredFrame();
  const { fps } = useAuthoredConfig();
  const isVertical = orientation === "vertical";
  const room = isVertical ? 980 : 1800;
  const span = logos.length * diameter - Math.max(0, logos.length - 1) * overlap;
  const fit = span > room ? room / span : 1;

  return (
    <AbsoluteFill
      style={{
        alignItems: "center",
        justifyContent: "center",
        background: "transparent",
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: isVertical ? "column" : "row",
          scale: `${fit}`,
        }}
      >
        {logos.map(({ image, background }, i) => {
          const s = spring({
            fps,
            frame: frame * speed - i * stagger,
            config: { damping: bounce, stiffness: 130, mass: 0.8 },
          });
          const scale = interpolate(s, [0, 1], [0.5, 1]);
          const offset = interpolate(s, [0, 1], [22, 0]);
          const lead = i === 0 ? 0 : -overlap;
          return (
            <div
              key={i}
              style={{
                width: diameter,
                height: diameter,
                borderRadius: "50%",
                marginLeft: isVertical ? 0 : lead,
                marginTop: isVertical ? lead : 0,
                zIndex: firstOnTop ? logos.length - i : i + 1,
                background,
                border: `${ringWidth}px solid ${ringColor}`,
                boxShadow: `0 20px 48px rgba(0,0,0,${shadow})`,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                opacity: s,
                translate: isVertical ? `${offset}px` : `0 ${offset}px`,
                scale: `${scale}`,
              }}
            >
              <div
                style={{
                  width: (diameter * logoScale) / 100,
                  height: (diameter * logoScale) / 100,
                  display: "flex",
                }}
              >
                {image ? (
                  <Img
                    src={image}
                    style={{ width: "100%", height: "100%", objectFit: "contain" }}
                  />
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
