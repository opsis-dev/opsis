import type { PortType } from "../schema/workflow";

const ACCEPTS: Record<PortType, PortType[]> = {
  session: ["session"],
  recording: ["recording"],
  clip: ["clip", "scene"],
  scene: ["clip", "scene"],
  timeline: ["timeline", "clip", "scene"],
  audio: ["audio", "media"],
  captions: ["captions"],
  image: ["image", "media"],
  media: ["media", "image", "file"],
  file: ["file", "media", "image"],
};

export function canConnect(source: PortType, target: PortType): boolean {
  return ACCEPTS[target]?.includes(source) ?? false;
}
