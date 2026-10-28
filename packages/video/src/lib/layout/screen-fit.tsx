import type React from "react";
import { useLayoutEffect, useRef, useState } from "react";
import { Sequence } from "remotion";

export const ScreenFit: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<{ width: number; height: number } | null>(null);
  useLayoutEffect(() => {
    const el = box.current;
    if (!el) return;
    const measure = () => {
      const width = Math.round(el.clientWidth);
      const height = Math.round(el.clientHeight);
      if (width > 0 && height > 0)
        setSize((prev) =>
          prev && prev.width === width && prev.height === height ? prev : { width, height },
        );
    };
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  return (
    <div ref={box} style={{ position: "absolute", inset: 0, overflow: "hidden" }}>
      {size ? (
        <Sequence layout="none" width={size.width} height={size.height}>
          {children}
        </Sequence>
      ) : null}
    </div>
  );
};
