import { Img } from "@opsis/runtime/parts.tsx";
import type { CSSProperties } from "react";

export function BrandImage({ src, style }: { src: string; style: CSSProperties }) {
  return <Img src={src} style={style} />;
}

export function OrbitMark({ color, size = 30 }: { color: string; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="-240 -240 480 480" fill="none" aria-hidden="true">
      <path
        d="M209.23 -67.98 A220 220 0 1 1 116.58 -186.57 L71.01 -113.64 A134 134 0 1 0 127.44 -41.41 Z"
        fill={color}
      />
    </svg>
  );
}
