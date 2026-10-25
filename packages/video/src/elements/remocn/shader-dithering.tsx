import { Dithering, type DitheringProps } from "@paper-design/shaders-react";
import { useCallback, useState } from "react";
import {
  continueRender,
  delayRender,
} from "remotion";
import { useAuthoredConfig, useAuthoredFrame } from "../../core/authored-clock";

export interface ShaderDitheringProps
  extends Omit<DitheringProps, "frame" | "ref"> {}

export function ShaderDithering({
  speed = 1,
  colorBack = "#12121a",
  colorFront = "#6a6a85",
  shape = "wave",
  type = "4x4",
  size = 2,
  className,
  ...rest
}: ShaderDitheringProps) {
  const frame = useAuthoredFrame();
  const { fps, width, height } = useAuthoredConfig();

  const [handle] = useState(() => delayRender("shader-dithering"));
  const gate = useCallback(
    (element: HTMLDivElement | null) => {
      if (!element) return;
      requestAnimationFrame(() =>
        requestAnimationFrame(() => continueRender(handle)),
      );
    },
    [handle],
  );

  return (
    <div
      ref={gate}
      className={className}
      style={{ position: "absolute", inset: 0 }}
    >
      <Dithering
        speed={0}
        frame={(frame / fps) * speed * 1000}
        colorBack={colorBack}
        colorFront={colorFront}
        shape={shape}
        type={type}
        size={size}
        fit="cover"
        width={width}
        height={height}
        {...rest}
      />
    </div>
  );
}
