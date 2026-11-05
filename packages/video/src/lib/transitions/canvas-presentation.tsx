import { useFilm, useFrame } from "@opsis/runtime/clock.tsx";
import { canDrawElementsInCanvas } from "@opsis/runtime/environment.ts";
import { holdRender } from "@opsis/runtime/holds.ts";
import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import React from "react";
import { createQuadSurface } from "#video/lib/shaders/quad.ts";
import type {
  TransitionPresentation,
  TransitionPresentationComponentProps,
} from "#video/lib/transitions/presentation.ts";
import { parseColor } from "#video/lib/ui-kit/color.ts";

const LAYER: React.CSSProperties = {
  position: "absolute",
  inset: 0,
  width: "100%",
  height: "100%",
};

const OUTPUT_LAYER: React.CSSProperties = { ...LAYER, pointerEvents: "none" };

function untilPainted(layout: HTMLCanvasElement, label: string): () => void {
  const release = holdRender(label);
  layout.addEventListener("paint", release, { once: true });
  layout.requestPaint?.();
  return () => {
    layout.removeEventListener("paint", release);
    release();
  };
}

function isCanvasTransitionSupported(): boolean {
  return canDrawElementsInCanvas();
}

function isCanvasFilterSupported(): boolean {
  return canDrawElementsInCanvas();
}

export type SceneShaderFrame<Props> = {
  gl: WebGL2RenderingContext;
  uniform: (name: string) => WebGLUniformLocation | null;
  width: number;
  height: number;
  progress: number;
  passedProps: Props;
};

type SceneShaderDrawParams<Props> = {
  from: OffscreenCanvas | null;
  to: OffscreenCanvas | null;
  width: number;
  height: number;
  progress: number;
  passedProps: Props;
};

type SceneShaderInstance<Props> = {
  draw: (params: SceneShaderDrawParams<Props>) => void;
  clear: () => void;
  cleanup: () => void;
};

export type SceneShader<Props> = (canvas: HTMLCanvasElement) => SceneShaderInstance<Props>;

export type SceneFilterFrame<Props> = {
  gl: WebGL2RenderingContext;
  uniform: (name: string) => WebGLUniformLocation | null;
  width: number;
  height: number;
  frame: number;
  time: number;
  passedProps: Props;
};

type SceneFilterDrawParams<Props> = {
  scene: OffscreenCanvas | null;
  width: number;
  height: number;
  frame: number;
  time: number;
  passedProps: Props;
};

type SceneFilterInstance<Props> = {
  draw: (params: SceneFilterDrawParams<Props>) => void;
  clear: () => void;
  cleanup: () => void;
};

export type SceneFilter<Props> = (canvas: HTMLCanvasElement) => SceneFilterInstance<Props>;

export function makeSceneShader<Props extends Record<string, unknown>>(
  fragment: string,
  setUniforms?: (frame: SceneShaderFrame<Props>) => void,
): SceneShader<Props> {
  return (canvas) => {
    const surface = createQuadSurface(canvas, fragment);
    const fromTexture = surface.createTexture();
    const toTexture = surface.createTexture();

    return {
      clear: surface.clear,
      cleanup: surface.cleanup,
      draw: ({ from, to, width, height, progress, passedProps }) => {
        surface.begin(width, height);
        surface.bindScene(from, fromTexture, 0, "u_from");
        surface.bindScene(to, toTexture, 1, "u_to");

        surface.gl.uniform1f(surface.uniform("u_progress"), progress);
        surface.gl.uniform1f(surface.uniform("u_aspect"), width / height);
        setUniforms?.({
          gl: surface.gl,
          uniform: surface.uniform,
          width,
          height,
          progress,
          passedProps,
        });

        surface.draw();
      },
    };
  };
}

export function makeFilterShader<Props extends Record<string, unknown>>(
  fragment: string,
  setUniforms?: (frame: SceneFilterFrame<Props>) => void,
): SceneFilter<Props> {
  return (canvas) => {
    const surface = createQuadSurface(canvas, fragment);
    const sceneTexture = surface.createTexture();

    return {
      clear: surface.clear,
      cleanup: surface.cleanup,
      draw: ({ scene, width, height, frame, time, passedProps }) => {
        surface.begin(width, height);
        surface.bindScene(scene, sceneTexture, 0, "u_scene");

        surface.gl.uniform1f(surface.uniform("u_frame"), frame);
        surface.gl.uniform1f(surface.uniform("u_time"), time);
        surface.gl.uniform1f(surface.uniform("u_aspect"), width / height);
        setUniforms?.({
          gl: surface.gl,
          uniform: surface.uniform,
          width,
          height,
          frame,
          time,
          passedProps,
        });

        surface.draw();
      },
    };
  };
}

const transferred = new WeakMap<HTMLCanvasElement, OffscreenCanvas>();

function takeOffscreen(canvas: HTMLCanvasElement): OffscreenCanvas {
  const existing = transferred.get(canvas);
  if (existing) {
    return existing;
  }
  const created = canvas.transferControlToOffscreen();
  transferred.set(canvas, created);
  return created;
}

function makeCanvasComponent<Props extends Record<string, unknown>>(
  shader: SceneShader<Props>,
): React.FC<TransitionPresentationComponentProps<Props>> {
  return function CanvasScenePresentation({
    children,
    presentationProgress,
    presentationDirection,
    passedProps,
    onElementImage,
    onUnmount,
    bothEnteringAndExiting,
  }) {
    const layoutRef = React.useRef<HTMLCanvasElement | null>(null);
    const outputRef = React.useRef<HTMLCanvasElement | null>(null);
    const captureRef = React.useRef<OffscreenCanvas | null>(null);
    const captureContextRef = React.useRef<OffscreenCanvasRenderingContext2D | null>(null);
    const outputSurfaceRef = React.useRef<HTMLCanvasElement | null>(null);
    const instanceRef = React.useRef<SceneShaderInstance<Props> | null>(null);
    const passedPropsRef = React.useRef(passedProps);
    passedPropsRef.current = passedProps;
    const onElementImageRef = React.useRef(onElementImage);
    onElementImageRef.current = onElementImage;
    const onUnmountRef = React.useRef(onUnmount);
    onUnmountRef.current = onUnmount;
    const passThrough = bothEnteringAndExiting && presentationDirection === "exiting";

    const drawRef = React.useRef<
      (
        prevImage: OffscreenCanvas | null,
        nextImage: OffscreenCanvas | null,
        progress: number,
      ) => void
    >(() => undefined);
    drawRef.current = (prevImage, nextImage, progress) => {
      const instance = instanceRef.current;
      if (!instance) {
        return;
      }
      const handle = holdRender("canvas-presentation: paint");
      const to = prevImage;
      const from = nextImage;
      const width = from?.width ?? to?.width ?? 0;
      const height = from?.height ?? to?.height ?? 0;
      if (width === 0 || height === 0) {
        instance.clear();
      } else {
        instance.draw({
          from,
          to,
          width,
          height,
          progress: !to ? 0 : !from ? 1 : progress,
          passedProps: passedPropsRef.current,
        });
      }
      handle();
    };

    const draw = React.useCallback(
      (prevImage: OffscreenCanvas | null, nextImage: OffscreenCanvas | null, progress: number) =>
        drawRef.current(prevImage, nextImage, progress),
      [],
    );

    React.useLayoutEffect(() => {
      if (passThrough) {
        return;
      }
      const layout = layoutRef.current;
      const output = outputRef.current;
      if (!layout || !output) {
        return;
      }
      const capture = takeOffscreen(layout);
      const context = capture.getContext("2d");
      if (!context) {
        throw new Error("canvas-presentation: failed to acquire a 2D context");
      }
      captureRef.current = capture;
      captureContextRef.current = context;
      outputSurfaceRef.current = output;
      instanceRef.current = shader(output);

      return () => {
        instanceRef.current?.cleanup();
        instanceRef.current = null;
        captureRef.current = null;
        captureContextRef.current = null;
        outputSurfaceRef.current = null;
      };
    }, [passThrough]);

    React.useLayoutEffect(() => {
      if (passThrough) {
        return;
      }
      const layout = layoutRef.current;
      if (!layout) {
        return;
      }
      layout.layoutSubtree = true;
      function handlePaint() {
        const painted = layoutRef.current;
        const scene = painted?.firstElementChild;
        const capture = captureRef.current;
        const context = captureContextRef.current;
        if (!painted || !scene || !capture || !context) {
          return;
        }
        const elementImage = painted.captureElementImage(scene);
        try {
          context.reset();
          context.drawElementImage(elementImage, 0, 0);
        } finally {
          elementImage.close();
        }
        onElementImageRef.current(capture, draw);
      }
      layout.addEventListener("paint", handlePaint);
      return () => layout.removeEventListener("paint", handlePaint);
    }, [passThrough, draw]);

    React.useLayoutEffect(() => {
      if (passThrough) {
        return;
      }
      const layout = layoutRef.current;
      if (!layout) {
        return;
      }
      const observer = new ResizeObserver(([entry]) => {
        const capture = captureRef.current;
        const surface = outputSurfaceRef.current;
        if (!capture || !surface) {
          return;
        }
        const width = entry.devicePixelContentBoxSize[0].inlineSize;
        const height = entry.devicePixelContentBoxSize[0].blockSize;
        capture.width = width;
        capture.height = height;
        surface.width = width;
        surface.height = height;
        layout.requestPaint?.();
      });
      observer.observe(layout, { box: "device-pixel-content-box" });
      return () => observer.disconnect();
    }, [passThrough]);

    React.useLayoutEffect(() => {
      const layout = layoutRef.current;
      if (passThrough || !layout) {
        return;
      }
      return untilPainted(layout, "canvas-presentation: paint");
    }, [passThrough, presentationProgress]);

    React.useLayoutEffect(() => {
      if (passThrough) {
        return;
      }
      return () => onUnmountRef.current();
    }, [passThrough]);

    if (passThrough) {
      return <>{children}</>;
    }

    return (
      <AbsoluteFill>
        <canvas ref={layoutRef} style={LAYER}>
          {children}
        </canvas>
        <canvas ref={outputRef} style={OUTPUT_LAYER} />
      </AbsoluteFill>
    );
  };
}

export type CanvasFilterProps = {
  children: React.ReactNode;
};

function makeFilterComponent<Props extends Record<string, unknown>>(
  shader: SceneFilter<Props>,
): React.FC<Props & CanvasFilterProps> {
  return function CanvasSceneFilter(props) {
    const frame = useFrame();
    const { fps } = useFilm();

    const layoutRef = React.useRef<HTMLCanvasElement | null>(null);
    const outputRef = React.useRef<HTMLCanvasElement | null>(null);
    const captureRef = React.useRef<OffscreenCanvas | null>(null);
    const captureContextRef = React.useRef<OffscreenCanvasRenderingContext2D | null>(null);
    const outputSurfaceRef = React.useRef<HTMLCanvasElement | null>(null);
    const instanceRef = React.useRef<SceneFilterInstance<Props> | null>(null);

    const passedPropsRef = React.useRef(props);
    passedPropsRef.current = props;
    const frameRef = React.useRef(frame);
    frameRef.current = frame;
    const timeRef = React.useRef(frame / fps);
    timeRef.current = frame / fps;

    React.useLayoutEffect(() => {
      const layout = layoutRef.current;
      const output = outputRef.current;
      if (!layout || !output) {
        return;
      }
      const capture = takeOffscreen(layout);
      const context = capture.getContext("2d");
      if (!context) {
        throw new Error("canvas-presentation: failed to acquire a 2D context");
      }
      captureRef.current = capture;
      captureContextRef.current = context;
      outputSurfaceRef.current = output;
      instanceRef.current = shader(output);

      return () => {
        instanceRef.current?.cleanup();
        instanceRef.current = null;
        captureRef.current = null;
        captureContextRef.current = null;
        outputSurfaceRef.current = null;
      };
    }, []);

    React.useLayoutEffect(() => {
      const layout = layoutRef.current;
      if (!layout) {
        return;
      }
      layout.layoutSubtree = true;
      function handlePaint() {
        const painted = layoutRef.current;
        const scene = painted?.firstElementChild;
        const capture = captureRef.current;
        const context = captureContextRef.current;
        const instance = instanceRef.current;
        if (!painted || !scene || !capture || !context || !instance) {
          return;
        }
        const elementImage = painted.captureElementImage(scene);
        try {
          context.reset();
          context.drawElementImage(elementImage, 0, 0);
        } finally {
          elementImage.close();
        }
        if (capture.width === 0 || capture.height === 0) {
          instance.clear();
        } else {
          instance.draw({
            scene: capture,
            width: capture.width,
            height: capture.height,
            frame: frameRef.current,
            time: timeRef.current,
            passedProps: passedPropsRef.current,
          });
        }
      }
      layout.addEventListener("paint", handlePaint);
      return () => layout.removeEventListener("paint", handlePaint);
    }, []);

    React.useLayoutEffect(() => {
      const layout = layoutRef.current;
      if (!layout) {
        return;
      }
      const observer = new ResizeObserver(([entry]) => {
        const capture = captureRef.current;
        const surface = outputSurfaceRef.current;
        if (!capture || !surface) {
          return;
        }
        const width = entry.devicePixelContentBoxSize[0].inlineSize;
        const height = entry.devicePixelContentBoxSize[0].blockSize;
        capture.width = width;
        capture.height = height;
        surface.width = width;
        surface.height = height;
        layout.requestPaint?.();
      });
      observer.observe(layout, { box: "device-pixel-content-box" });
      return () => observer.disconnect();
    }, []);

    React.useLayoutEffect(() => {
      const layout = layoutRef.current;
      if (!layout) {
        return;
      }
      return untilPainted(layout, "canvas-presentation: filter paint");
    });

    return (
      <AbsoluteFill>
        <canvas ref={layoutRef} style={LAYER}>
          {props.children}
        </canvas>
        <canvas ref={outputRef} style={OUTPUT_LAYER} />
      </AbsoluteFill>
    );
  };
}

export type CanvasPresentationOptions<Props extends Record<string, unknown>> = {
  shader: SceneShader<Props>;
  fallback: React.FC<TransitionPresentationComponentProps<Props>>;
};

export function makeCanvasPresentation<Props extends Record<string, unknown>>({
  shader,
  fallback: Fallback,
}: CanvasPresentationOptions<Props>): (props: Props) => TransitionPresentation<Props> {
  const Canvas = makeCanvasComponent(shader);

  function Presentation(props: TransitionPresentationComponentProps<Props>) {
    return isCanvasTransitionSupported() ? <Canvas {...props} /> : <Fallback {...props} />;
  }

  return (props) => ({ component: Presentation, props });
}

export type CanvasFilterOptions<Props extends Record<string, unknown>> = {
  shader: SceneFilter<Props>;
  fallback: React.FC<Props & CanvasFilterProps>;
};

export function makeCanvasFilter<Props extends Record<string, unknown>>({
  shader,
  fallback: Fallback,
}: CanvasFilterOptions<Props>): React.FC<Props & CanvasFilterProps> {
  const Canvas = makeFilterComponent(shader);

  return function CanvasFilter(props) {
    return isCanvasFilterSupported() ? <Canvas {...props} /> : <Fallback {...props} />;
  };
}

export function rgbUniform(color: string): [number, number, number] {
  const { r, g, b } = parseColor(color);
  return [r, g, b];
}
