import React, { useEffect, useState } from "react";
import { AbsoluteFill, cancelRender, continueRender, delayRender } from "remotion";
import { Lottie, type LottieAnimationData } from "@remotion/lottie";
import { visibleBackground } from "./anim";
import { readLottie } from "./dot-lottie";

type Fit = "contain" | "cover";

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
  fit?: Fit;
  background?: string;
}> = ({ src, loop = true, speed = 1, backwards = false, fit = "contain", background }) => {
  const [data, setData] = useState<LottieAnimationData | null>(null);
  const [handle] = useState(() => (src ? delayRender(`Loading animation ${src}`) : null));

  useEffect(() => {
    if (!src || handle === null) return;
    let alive = true;
    loadAnimation(src)
      .then((d) => {
        if (!alive) return;
        setData(d);
        continueRender(handle);
      })
      .catch((err: unknown) => cancelRender(err));
    return () => {
      alive = false;
    };
  }, [src, handle]);

  if (!src || !data) return null;
  return (
    <AbsoluteFill style={{ background: visibleBackground(background) }}>
      <Lottie
        animationData={data}
        loop={loop}
        playbackRate={speed}
        direction={backwards ? "backward" : "forward"}
        preserveAspectRatio={fit === "cover" ? "xMidYMid slice" : "xMidYMid meet"}
        style={{ width: "100%", height: "100%" }}
      />
    </AbsoluteFill>
  );
};
