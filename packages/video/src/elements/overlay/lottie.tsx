import { Lottie, type LottieAnimationData } from "@remotion/lottie";
import { readLottie } from "#video/core/dot-lottie.ts";
import { useHeldLoad } from "#video/core/loading.ts";
import type { ElementSettings, TextAlign } from "#video/generated/spec.ts";
import { Backgrounded } from "#video/lib/backgrounds/backgrounded.tsx";

const ALIGN = {
  left: "xMinYMid",
  center: "xMidYMid",
  right: "xMaxYMid",
} as const satisfies Record<TextAlign, string>;

async function loadAnimation(src: string): Promise<LottieAnimationData> {
  const res = await fetch(src);
  if (!res.ok) throw new Error(`Couldn't load the animation (${res.status}).`);
  return readLottie(new Uint8Array(await res.arrayBuffer()));
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
      <Lottie
        animationData={data}
        loop={loop}
        playbackRate={speed}
        direction={backwards ? "backward" : "forward"}
        preserveAspectRatio={`${ALIGN[align]} ${fit === "cover" ? "slice" : "meet"}`}
        style={{ width: "100%", height: "100%" }}
      />
    </Backgrounded>
  );
}
