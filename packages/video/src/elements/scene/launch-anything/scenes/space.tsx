import { Img } from "@opsis/runtime/parts.tsx";
import { type SceneProps, shade } from "#video/elements/scene/launch-anything/content.ts";
import { tween } from "#video/elements/scene/launch-anything/motion.ts";
import { Center, Title } from "#video/elements/scene/launch-anything/ui.tsx";

export function Space({ scene, t }: SceneProps) {
  const first = t < 20.62;
  const inset = !first && t < 21.02;
  const age = t - (first ? 20.22 : inset ? 20.62 : 21.02);
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        overflow: "hidden",
        background: shade(scene.palette.paper, 5),
      }}
    >
      <Img
        src={first ? scene.media.rocket : scene.media.earth}
        style={{
          position: "absolute",
          inset: inset ? "20px 32px" : 0,
          width: inset ? 416 : "100%",
          height: inset ? 230 : "100%",
          objectFit: "cover",
          transform: `translateX(${tween(age, 0, 0.53, -3, 3)}px) scale(${tween(age, 0, 0.53, 1.07, 1.02)})`,
          filter: `brightness(${(scene.look.photoDimming * (first ? 0.8 : 0.68)) / 0.66})`,
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 18,
          border: "0.65px solid #f4f0e970",
          opacity: inset ? 0 : 0.8,
        }}
      />
      <Center>
        <Title
          scene={scene}
          text={scene.content.launch}
          size={92}
          style={{
            letterSpacing: "-0.05em",
            fontWeight: Math.max(100, scene.look.headlineWeight - 100),
            color: scene.palette.lightText,
          }}
        />
      </Center>
    </div>
  );
}
