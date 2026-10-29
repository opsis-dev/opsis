import { Img, Sequence, useVideoConfig } from "remotion";
import type { SceneProps } from "#video/elements/scene/launch-anything/content.ts";
import { tween } from "#video/elements/scene/launch-anything/motion.ts";
import { ShaderTextReveal } from "#video/elements/scene/launch-anything/shader-text-reveal.tsx";
import { Center, Title } from "#video/elements/scene/launch-anything/ui.tsx";

export function Opening({ scene, t }: SceneProps) {
  const { fps } = useVideoConfig();
  return (
    <>
      <Sequence name="Opening — chrome reveal" durationInFrames={Math.round(fps)}>
        <div
          style={{
            position: "absolute",
            inset: 0,
            filter: "sepia(0.7) hue-rotate(65deg) saturate(0.65) brightness(1.35)",
            maskImage: `linear-gradient(125deg, transparent ${tween(t, 0.7, 1, -70, 105)}%, black ${tween(t, 0.7, 1, -45, 130)}%)`,
          }}
        >
          <ShaderTextReveal
            text={`${scene.content.opening}\n`}
            fontFamily={scene.look.font}
            fontWeight={500}
            fontSize={scene.look.openingSize}
            intensity={scene.look.openingShimmer}
            wordDuration={fps * 1.4}
          />
        </div>
      </Sequence>
      <Sequence
        name="Subject — chrome reveal"
        from={Math.round(fps)}
        durationInFrames={Math.round(fps * 1.5)}
      >
        <div
          style={{
            position: "absolute",
            inset: 0,
            filter: "sepia(0.7) hue-rotate(65deg) saturate(0.65) brightness(1.35)",
          }}
        >
          <ShaderTextReveal
            text={`${scene.content.subject}\n`}
            fontFamily={scene.look.font}
            fontWeight={500}
            fontSize={(scene.look.openingSize * 300) / 430}
            intensity={scene.look.openingShimmer}
            wordDuration={fps * 1.4}
          />
        </div>
      </Sequence>
      {t >= 2.5 ? (
        <>
          <Img
            src={scene.media.horizon}
            style={{
              position: "absolute",
              inset: 0,
              width: "100%",
              height: "100%",
              objectFit: "cover",
              opacity: tween(t, 2.5, 2.85),
              transform: `scale(${tween(t, 2.5, 3.8, 1.1, 1)})`,
              filter: `brightness(${scene.look.photoDimming}) saturate(0.85) blur(0.4px)`,
            }}
          />
          <Center>
            <Title
              scene={scene}
              text={scene.content.ready}
              size={57}
              style={{
                color: scene.palette.lightText,
                opacity: tween(t, 2.5, 2.7),
                filter: `blur(${tween(t, 2.5, 2.9, 7, 0)}px)`,
                textShadow: "0 0 3px #fff6",
              }}
            />
          </Center>
        </>
      ) : null}
    </>
  );
}
