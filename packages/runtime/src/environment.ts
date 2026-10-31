import { getRemotionEnvironment, isHtmlInCanvasSupported } from "remotion";

export function isRendering(): boolean {
  return getRemotionEnvironment().isRendering;
}

export function canDrawElementsInCanvas(): boolean {
  return isHtmlInCanvasSupported();
}
