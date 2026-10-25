import { Lottie, type LottieAnimationData } from "@remotion/lottie";
import React from "react";
import { Backgrounded } from "./backgrounds";
import { readLottie } from "./dot-lottie";
import { useHeldLoad } from "./loading";

type LottieFit = "contain" | "cover";

async function loadAnimation(src: string): Promise<LottieAnimationData> {
  const res = await fetch(src);
  if (!res.ok) throw new Error(`Couldn't load the animation (${res.status}).`);
  return readLottie(new Uint8Array(await res.arrayBuffer()));
}

export const LottieElement: React.FC<{
  src: string;
  loop?: boolean;
  speed?: number;
  backwards?: boolean;
  fit?: LottieFit;
  background?: string;
}> = ({ src, loop = true, speed = 1, backwards = false, fit = "contain", background }) => {
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
        preserveAspectRatio={fit === "cover" ? "xMidYMid slice" : "xMidYMid meet"}
        style={{ width: "100%", height: "100%" }}
      />
    </Backgrounded>
  );
};
