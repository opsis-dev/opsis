import { Img } from "remotion";
import {
  type LaunchScene,
  type SceneProps,
  shade,
  tint,
} from "#video/elements/scene/launch-anything/content.ts";
import { actionFrame, smooth, tween } from "#video/elements/scene/launch-anything/motion.ts";

export function actionBackground(scene: LaunchScene) {
  return `linear-gradient(${shade(scene.palette.paper, 3)}, ${scene.palette.paper} 75%)`;
}

export function ActionFrame({ scene, t }: SceneProps) {
  const shape = actionFrame(t);
  const aperture = tween(t, 6.7, 7.33, 0, 1, smooth);
  const { paper, sage } = scene.palette;
  return (
    <div
      style={{
        position: "absolute",
        left: shape.x - shape.width / 2,
        top: shape.y - shape.height / 2,
        width: shape.width,
        height: shape.height,
        borderRadius: shape.radius,
        opacity: shape.enter,
        transform: `scale(${1 - 0.07 * shape.press})`,
        overflow: "hidden",
        background: `linear-gradient(135deg, ${paper} 10%, ${shade(paper, 2)} 36%, ${tint(sage, 76)} 70%, ${tint(sage, 38)} 100%)`,
        boxShadow: "inset 1px 1px 2px #fff, 0 4px 18px #728b6415",
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          opacity: shape.morph,
          background: `linear-gradient(140deg, ${tint(sage, 35)}, ${tint(sage, 72)} 35%, ${shade(paper, 1)} 60%, ${tint(sage, 42)})`,
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: shade(scene.palette.ink, 88),
          fontSize: Math.min(36, 320 / Math.max(1, Array.from(scene.content.action).length)),
          letterSpacing: "-0.045em",
          fontWeight: 500,
          whiteSpace: "nowrap",
          opacity: 1 - tween(t, 5.55, 5.83),
        }}
      >
        {scene.content.action}
      </div>
      <div
        style={{
          position: "absolute",
          inset: 38 * (1 - aperture) * shape.morph,
          borderRadius: 13 + 33 * shape.morph,
          border: `${22 * (1 - aperture) * shape.morph}px solid #fff`,
          borderRightWidth: 34 * (1 - aperture) * shape.morph,
          borderBottomWidth: 30 * (1 - aperture) * shape.morph,
          boxSizing: "border-box",
          opacity: shape.morph,
          overflow: "hidden",
          background: `linear-gradient(140deg, ${shade(paper, 1)} 25%, ${shade(paper, 4)} 46%, ${tint(sage, 72)} 72%, ${tint(sage, 32)})`,
          boxShadow: "inset 0 0 12px #ffffff9a",
        }}
      >
        <div
          style={{
            position: "absolute",
            inset: 0,
            background: `linear-gradient(120deg, ${tint(sage, 70)} 6%, ${tint(sage, 35)} 27%, ${scene.accent} 51%, ${tint(sage, 20)} 75%, ${tint(sage, 82)})`,
            opacity: tween(t, 6.85, 7.07),
          }}
        />
        {t >= 7.08 ? (
          <Img
            src={scene.media.desk}
            style={{
              position: "absolute",
              inset: 0,
              width: "100%",
              height: "100%",
              objectFit: "cover",
              opacity: tween(t, 7.08, 7.32),
              transform: "scale(1.02)",
            }}
          />
        ) : null}
      </div>
    </div>
  );
}
