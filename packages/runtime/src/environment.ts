declare global {
  interface DrawElementImageOptions {
    preserveElementGeometry?: boolean;
  }
  interface ElementImage {
    readonly width: number;
    readonly height: number;
    close(): void;
  }
  interface CanvasRenderingContext2D {
    drawElementImage(
      element: Element | ElementImage,
      dx: number,
      dy: number,
      options?: DrawElementImageOptions,
    ): void;
  }
  interface OffscreenCanvasRenderingContext2D {
    drawElementImage(
      element: Element | ElementImage,
      dx: number,
      dy: number,
      options?: DrawElementImageOptions,
    ): void;
  }
  interface HTMLCanvasElementEventMap {
    paint: Event;
  }
  interface HTMLCanvasElement {
    layoutSubtree?: boolean;
    requestPaint?(): void;
    captureElementImage(element: Element): ElementImage;
  }
}

let rendering = false;
let drawsElements: boolean | null = null;

export function markRendering(): void {
  rendering = true;
}

export function isRendering(): boolean {
  return rendering;
}

export function canDrawElementsInCanvas(): boolean {
  if (drawsElements !== null) return drawsElements;
  if (typeof document === "undefined") return false;
  const canvas = document.createElement("canvas");
  const context = canvas.getContext("2d");
  drawsElements =
    typeof context?.drawElementImage === "function" &&
    typeof canvas.requestPaint === "function" &&
    typeof canvas.captureElementImage === "function" &&
    "transferControlToOffscreen" in HTMLCanvasElement.prototype;
  return drawsElements;
}
