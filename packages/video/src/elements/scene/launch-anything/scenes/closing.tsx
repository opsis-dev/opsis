import { clamp01 } from "#video/common/math.ts";
import { type SceneProps, shade, tint } from "#video/elements/scene/launch-anything/content.ts";
import { smooth, tween } from "#video/elements/scene/launch-anything/motion.ts";
import { BrandMark, Center, Title } from "#video/elements/scene/launch-anything/ui.tsx";

export function Address({ scene, t }: SceneProps) {
  const url = scene.brandUrl;
  const prefixLength = url.toLowerCase().startsWith(scene.content.launch.toLowerCase())
    ? scene.content.launch.length
    : 0;
  const typing = clamp01((t - 21.94) / 0.59);
  const text = url.slice(0, prefixLength + Math.floor((url.length - prefixLength) * typing));
  const glow = tween(t, 22.15, 22.45) * (1 - tween(t, 22.6, 23.25));
  return (
    <Center style={{ background: scene.palette.night, color: scene.palette.lightText }}>
      <Title
        scene={scene}
        text={text}
        size={tween(t, 21.55, 21.9, 43, 32, smooth)}
        style={{
          fontWeight: 400,
          letterSpacing: "-0.06em",
          color: glow > 0.5 ? tint(scene.palette.sage, 75) : scene.palette.lightText,
          textShadow: `0 0 ${glow * 5}px #fff, 0 0 ${glow * 20}px ${tint(scene.palette.sage, 25)}, 0 0 ${glow * 32}px ${scene.palette.sage}`,
        }}
      />
    </Center>
  );
}

export function ClosingMark({ scene, t }: SceneProps) {
  return (
    <Center
      style={{
        background: `linear-gradient(${shade(scene.palette.paper, 2.5)}, ${scene.palette.paper})`,
        color: scene.accent,
      }}
    >
      <div
        style={{
          transform: `scale(${tween(t, 23.717, 26.666, 1, 0.63, smooth)})`,
        }}
      >
        <BrandMark size={scene.look.logoSize} src={scene.logoSrc} />
      </div>
    </Center>
  );
}
