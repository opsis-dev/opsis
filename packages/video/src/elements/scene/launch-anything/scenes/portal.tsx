import type { SceneProps } from "#video/elements/scene/launch-anything/content.ts";
import {
  ActionFrame,
  actionBackground,
} from "#video/elements/scene/launch-anything/scenes/action-frame.tsx";
export function Portal(props: SceneProps) {
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: actionBackground,
        overflow: "hidden",
      }}
    >
      <ActionFrame {...props} />
    </div>
  );
}
