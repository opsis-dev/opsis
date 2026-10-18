import type { ClipValue } from "../schema/values";
import type { PortType } from "../schema/workflow";
import { clipInput, opt, type CompileInputs, type PortSpec } from "./define";

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
  return ACCEPTS[target].includes(source);
}

export const SESSION_PORT = "session";

export const IN_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"] as const;

export type InPort = (typeof IN_PORTS)[number];

export const [, ...CUT_PORTS] = IN_PORTS;

export const inPortIndex = (handle: string): number => IN_PORTS.findIndex((p) => p === handle);

export function nextInPort(handle: string): InPort | undefined {
  const index = inPortIndex(handle);
  return index === -1 ? undefined : IN_PORTS[index + 1];
}

export const clipLabel = (handle: InPort) => `Clip ${inPortIndex(handle) + 1}`;

const LAYER_ENDS: Partial<Record<InPort, string>> = { in1: " (back)", in6: " (front)" };

export const layerLabel = (handle: InPort) =>
  `Layer ${inPortIndex(handle) + 1}${LAYER_ENDS[handle] ?? ""}`;

export const sceneLabel = (handle: InPort) => `Scene ${inPortIndex(handle) + 1}`;

export const clipPorts = (label: (handle: InPort) => string): Record<string, PortSpec> =>
  Object.fromEntries(IN_PORTS.map((p) => [p, opt("clip", label(p))]));

export const connectedClips = (inputs: CompileInputs): Array<{ port: InPort; clip: ClipValue }> =>
  IN_PORTS.flatMap((name) => {
    const clip = clipInput(inputs, name);
    return clip ? [{ port: name, clip }] : [];
  });
