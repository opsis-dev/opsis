import type React from "react";
import { useLayoutEffect, useRef, useState } from "react";
import { AbsoluteFill, Sequence, useVideoConfig } from "remotion";
import type { ElementProps } from "../../shared/elements/types";
import { numberOr } from "../../shared/math";
import type { Size } from "../core/still-layout";
import type { ElementComponent, Presentation, PresentationFactory } from "./types";

export const component = (element: React.FC<never>): ElementComponent =>
  element as ElementComponent;

export const presentation = (make: (props: never, size: Size) => unknown): PresentationFactory => ({
  create: make as (props: ElementProps, size: Size) => Presentation,
});

export function centered(Component: ElementComponent): ElementComponent {
  const Centered: ElementComponent = (props) => (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", textAlign: "center" }}>
      <Component {...props} />
    </AbsoluteFill>
  );
  return Centered;
}

const ScreenFit: React.FC<{ children: React.ReactNode }> = ({ children }) => {
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

export function fitScreen(Component: ElementComponent): ElementComponent {
  const Fitted: ElementComponent = ({ children, ...props }) => (
    <Component {...props}>{children ? <ScreenFit>{children}</ScreenFit> : undefined}</Component>
  );
  return Fitted;
}

export function placedAt(Component: ElementComponent): ElementComponent {
  const Placed: ElementComponent = ({ x, y, ...props }) => {
    const { width, height } = useVideoConfig();
    const left = numberOr(x, 0.5) * width - numberOr(props.width, 0) / 2;
    const top = numberOr(y, 0.5) * height - numberOr(props.height, 0) / 2;
    return (
      <AbsoluteFill>
        <div style={{ position: "absolute", left, top }}>
          <Component {...props} />
        </div>
      </AbsoluteFill>
    );
  };
  return Placed;
}

export function pointsFromSpots(Component: ElementComponent): ElementComponent {
  const Pointed: ElementComponent = ({ fromX, fromY, toX, toY, ...props }) => {
    const { width, height } = useVideoConfig();
    const point = (x: unknown, y: unknown) => ({
      x: numberOr(x, 0.5) * width,
      y: numberOr(y, 0.5) * height,
    });
    return (
      <AbsoluteFill>
        <Component {...props} from={point(fromX, fromY)} to={point(toX, toY)} />
      </AbsoluteFill>
    );
  };
  return Pointed;
}

const PAINT_RULE = ".df-painted > div { background: var(--df-paint) !important; }";

export const PaintRule: React.FC = () => <style>{PAINT_RULE}</style>;

export function painted(Component: ElementComponent): ElementComponent {
  const Painted: ElementComponent = ({ background, ...props }) => (
    <AbsoluteFill
      className="df-painted"
      style={{ "--df-paint": String(background ?? "transparent") } as React.CSSProperties}
    >
      <Component {...props} />
    </AbsoluteFill>
  );
  return Painted;
}

export function playsAtEnd(Component: ElementComponent): ElementComponent {
  const AtEnd: ElementComponent = (props) => {
    const { durationInFrames } = useVideoConfig();
    const length = numberOr(props.durationInFrames, 0);
    return <Component {...props} delay={Math.max(0, durationInFrames - length)} />;
  };
  return AtEnd;
}
