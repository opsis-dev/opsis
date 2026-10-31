import { Img } from "@opsis/runtime/parts.tsx";
import { type SceneProps, shade, tint } from "#video/elements/scene/launch-anything/content.ts";
import { tween } from "#video/elements/scene/launch-anything/motion.ts";
import { Center, LogoGlyph, Title } from "#video/elements/scene/launch-anything/ui.tsx";

const positions = [
  [26, -6],
  [107, 6],
  [190, -8],
  [282, 2],
  [371, -8],
  [455, 5],
  [43, 65],
  [114, 76],
  [188, 55],
  [253, 72],
  [317, 58],
  [393, 73],
  [465, 61],
  [34, 135],
  [443, 135],
  [35, 203],
  [109, 211],
  [183, 199],
  [253, 223],
  [315, 199],
  [387, 211],
  [459, 200],
  [28, 277],
  [118, 266],
  [198, 284],
  [286, 266],
  [375, 283],
  [468, 270],
];

export function Proof({ scene, t }: SceneProps) {
  const age = t - 907 / 60;
  const reveal = tween(age, 0.06, 0.73);
  const zoom = tween(age, 0.13, 1.1, 1.43, 1);
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: `linear-gradient(${shade(scene.palette.paper, 3)}, ${scene.palette.paper} 65%, ${shade(scene.palette.paper, 5)})`,
        color: scene.palette.ink,
        overflow: "hidden",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          transform: `scale(${zoom}) translateX(${(1 - reveal) * 340}px)`,
          transformOrigin: "50% 50%",
        }}
      >
        {positions.map(([x, y], i) => {
          const blur = [0.2, 1.3, 0, 0.7, 0.1, 1.7][i % 6];
          const src = scene.logos.length ? scene.logos[i % scene.logos.length] : undefined;
          return (
            <div
              key={i}
              style={{
                position: "absolute",
                left: x - 12,
                top: y - 12 + Math.sin(age * 0.7 + i) * 2,
                width: 27,
                height: 27,
                display: "grid",
                placeItems: "center",
                color: scene.palette.sage,
                opacity: [0.73, 0.3, 0.58, 0.19, 0.4][i % 5] * reveal,
                filter: `blur(${blur}px)`,
                transform: `rotate(${Math.sin(i * 2.8) * 9}deg)`,
                background: i % 7 === 0 ? tint(scene.palette.sage, 78) : undefined,
                borderRadius: 7,
                boxShadow: i % 7 === 0 ? "0 3px 12px #34492812" : undefined,
              }}
            >
              {src ? (
                <Img src={src} style={{ width: 27, height: 27, objectFit: "contain" }} />
              ) : (
                <LogoGlyph variant={i * 5 + 2} size={24} />
              )}
            </div>
          );
        })}
        <Center>
          <Title
            scene={scene}
            text={scene.content.proof}
            size={32}
            style={{
              fontWeight: 500,
              letterSpacing: "-0.033em",
              opacity: reveal,
              filter: `blur(${(1 - reveal) * 5}px)`,
            }}
          />
        </Center>
      </div>
    </div>
  );
}
