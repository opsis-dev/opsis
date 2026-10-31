import { holdRender, useHeldLoad } from "@opsis/runtime/holds.ts";
import lottie, { type AnimationItem } from "lottie-web/build/player/lottie_svg";
import { useLayoutEffect, useRef } from "react";
import { cancelRender, useCurrentFrame, useVideoConfig } from "remotion";
import {
  animationFrame,
  type LottieAnimation,
  type LottieTiming,
  readLottie,
} from "#video/core/dot-lottie.ts";
import type { ElementSettings, TextAlign } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

const ALIGN = {
  left: "xMinYMid",
  center: "xMidYMid",
  right: "xMaxYMid",
} as const satisfies Record<TextAlign, string>;

async function loadAnimation(src: string): Promise<LottieAnimation> {
  const res = await fetch(src);
  if (!res.ok) throw new Error(`Couldn't load the animation (${res.status}).`);
  return readLottie(new Uint8Array(await res.arrayBuffer()));
}

function hasImages(data: LottieAnimation): boolean {
  return (data.assets ?? []).some((asset) => asset.p !== undefined);
}

type Mount = {
  container: HTMLDivElement;
  data: LottieAnimation;
  aspect: string;
  wanted: { current: number };
};

function mount({ container, data, aspect, wanted }: Mount): AnimationItem {
  const release = holdRender("Drawing the animation");
  const waiting = new Set<string>(hasImages(data) ? ["DOMLoaded", "loaded_images"] : ["DOMLoaded"]);
  const animation = lottie.loadAnimation({
    container,
    renderer: "svg",
    loop: false,
    autoplay: false,
    animationData: structuredClone(data),
    rendererSettings: { preserveAspectRatio: aspect },
  });
  function arrived(name: string) {
    waiting.delete(name);
    if (waiting.size === 0) release();
  }
  animation.addEventListener("DOMLoaded", () => {
    animation.goToAndStop(wanted.current, true);
    arrived("DOMLoaded");
  });
  animation.addEventListener("loaded_images", () => arrived("loaded_images"));
  animation.addEventListener("data_failed", () =>
    cancelRender(new Error("Couldn't read the animation.")),
  );
  animation.addEventListener("destroy", release);
  return animation;
}

function LottiePlayer({
  data,
  aspect,
  timing,
}: {
  data: LottieAnimation;
  aspect: string;
  timing: Omit<LottieTiming, "frame" | "fps">;
}) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const container = useRef<HTMLDivElement>(null);
  const animation = useRef<AnimationItem | null>(null);
  const at = animationFrame(data, { ...timing, frame, fps });
  const wanted = useRef(at);
  useLayoutEffect(() => {
    const target = container.current;
    if (!target) return;
    const made = mount({ container: target, data, aspect, wanted });
    animation.current = made;
    return () => {
      animation.current = null;
      made.destroy();
    };
  }, [data, aspect]);
  useLayoutEffect(() => {
    wanted.current = at;
    animation.current?.goToAndStop(at, true);
  }, [at]);
  return <div ref={container} style={{ width: "100%", height: "100%" }} />;
}

export function LottieElement({
  src,
  loop,
  speed,
  backwards,
  fit,
  background,
  align,
}: ElementSettings["lottie"]) {
  const loaded = useHeldLoad(`Loading animation ${src}`, src || null, loadAnimation);
  const data = loaded?.key === src ? loaded.value : null;
  if (!src || !data) return null;
  return (
    <Backgrounded background={background}>
      <LottiePlayer
        data={data}
        aspect={`${ALIGN[align]} ${fit === "cover" ? "slice" : "meet"}`}
        timing={{ speed, loop, backwards }}
      />
    </Backgrounded>
  );
}
