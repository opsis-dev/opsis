import {
  AbsoluteFill,
  Audio,
  Sequence,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { type LaunchAnythingProps, resolveLaunchProps } from "./content";
import {
  LAUNCH_FPS,
  launchTimeline,
} from "./motion";
import { Action } from "./scenes/action";
import { Address, ClosingMark } from "./scenes/closing";
import { Industry } from "./scenes/industry";
import { Opening } from "./scenes/opening";
import { Portal } from "./scenes/portal";
import { Proof } from "./scenes/proof";
import { Showcase } from "./scenes/showcase";
import { Space } from "./scenes/space";
import type { ElementProps } from "#video/common/spec.ts";
import { listsFrom, nestedFrom } from "#video/lib/settings/shape.ts";

export { launchMedia } from "./assets";
export type {
  LaunchAnythingProps,
  LaunchContent,
  LaunchMedia,
} from "./content";
export { launchContent } from "./content";
export {
  LAUNCH_FPS,
  launchTimeline,
  showcaseTimeline,
} from "./motion";

const scenes = {
  opening: Opening,
  action: Action,
  portal: Portal,
  showcase: Showcase,
  proof: Proof,
  industry: Industry,
  space: Space,
  address: Address,
  mark: ClosingMark,
};

function LaunchAnythingView(props: LaunchAnythingProps) {
  const frame = useCurrentFrame();
  const { width, height, fps } = useVideoConfig();
  const scene = resolveLaunchProps(props);
  const scale = Math.min(width / 480, height / 270);
  const volume = Number.isFinite(props.volume)
    ? Math.min(1, Math.max(0, props.volume ?? 1))
    : 1;
  return (
    <AbsoluteFill
      name="Product Showcase"
      style={{
        background: "#122d24",
        color: "#fff",
        overflow: "hidden",
        fontFamily: "Manrope, sans-serif",
        WebkitFontSmoothing: "antialiased",
      }}
    >
      <div
        style={{
          position: "absolute",
          left: (width - 480 * scale) / 2,
          top: (height - 270 * scale) / 2,
          width: 480,
          height: 270,
          transform: `scale(${scale})`,
          transformOrigin: "0 0",
          overflow: "hidden",
        }}
      >
        {launchTimeline.map((shot) => {
          const Component = scenes[shot.id];
          return (
            <Sequence
              key={shot.id}
              name={shot.id}
              from={Math.round((shot.from / LAUNCH_FPS) * fps)}
              durationInFrames={
                Math.round((shot.to / LAUNCH_FPS) * fps) -
                Math.round((shot.from / LAUNCH_FPS) * fps)
              }
              layout="none"
            >
              <Component scene={scene} t={frame / fps} />
            </Sequence>
          );
        })}
      </div>
      {props.audioSrc?.trim() ? (
        <Audio src={props.audioSrc} volume={volume} />
      ) : null}
    </AbsoluteFill>
  );
}

export function LaunchAnything(props: ElementProps) {
  const settings = nestedFrom(listsFrom(props, {"logoImages":{"image":"logos"}}), {"copyReady":"content.ready","copyNext":"content.next","copyAction":"content.action","copyProof":"content.proof","copyIndustry":"content.industry","copyPromise":"content.promise","copyLaunch":"content.launch","copyShoppers":"content.shoppers","copyShopName":"content.shopName","copyShopHeadline":"content.shopHeadline","copyShopDetail":"content.shopDetail","copyShopAction":"content.shopAction","copyBuilder":"content.builder","copyBuilderPrompt":"content.builderPrompt","copyTrading":"content.trading","copyLoyalty":"content.loyalty","copyLoyaltyPayoff":"content.loyaltyPayoff","copyIntegration":"content.integration","copyConnections":"content.connections","copyIntegrations":"content.integrations","screenDashboard":"screenImages.dashboard","screenShoppers":"screenImages.shoppers","screenExchange":"screenImages.exchange","screenBuilder":"screenImages.builder","screenTrading":"screenImages.trading","screenLoyalty":"screenImages.loyalty","screenIntegrations":"screenImages.integrations","photoDesk":"media.desk","photoHorizon":"media.horizon","photoRocket":"media.rocket","photoEarth":"media.earth"});
  return <LaunchAnythingView {...(settings as unknown as LaunchAnythingProps)} />;
}
