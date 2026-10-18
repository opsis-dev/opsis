import { Lottie, type LottieAnimationData } from "@remotion/lottie";
import React, { useLayoutEffect, useState } from "react";
import { cancelRender } from "remotion";
import { Backgrounded } from "./backgrounds";
import { readLottie } from "./dot-lottie";
import { holdRender } from "./loading";

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
  const [loaded, setLoaded] = useState<{ src: string; data: LottieAnimationData } | null>(null);

  useLayoutEffect(() => {
    if (!src) return;
    const release = holdRender(`Loading animation ${src}`);
    let alive = true;
    loadAnimation(src)
      .then((data) => {
        if (!alive) return;
        setLoaded({ src, data });
        release();
      })
      .catch((err: unknown) => cancelRender(err));
    return () => {
      alive = false;
      release();
    };
  }, [src]);

  const data = loaded?.src === src ? loaded.data : null;
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
