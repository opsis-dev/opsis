import { type CSSProperties, type ReactNode, useLayoutEffect, useRef } from "react";
import { failRender, holdRender } from "#runtime/holds.ts";

const FILL: CSSProperties = {
  position: "absolute",
  top: 0,
  left: 0,
  right: 0,
  bottom: 0,
  width: "100%",
  height: "100%",
  display: "flex",
  flexDirection: "column",
};

export function AbsoluteFill({
  style,
  className,
  children,
}: {
  style?: CSSProperties;
  className?: string;
  children?: ReactNode;
}) {
  return (
    <div className={className} style={{ ...FILL, ...style }}>
      {children}
    </div>
  );
}

export function Img({
  src,
  alt = "",
  style,
  width,
  height,
}: {
  src: string;
  alt?: string;
  style?: CSSProperties;
  width?: number;
  height?: number;
}) {
  const image = useRef<HTMLImageElement>(null);
  useLayoutEffect(() => {
    const target = image.current;
    if (!target) return;
    const release = holdRender(`Loading ${src}`);
    let alive = true;
    target.decode().then(release, () => {
      if (alive) failRender(new Error(`Couldn't load the image ${src}.`));
    });
    return () => {
      alive = false;
      release();
    };
  }, [src]);
  return <img ref={image} src={src} alt={alt} style={style} width={width} height={height} />;
}
