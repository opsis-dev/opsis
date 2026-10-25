import { humanize } from "#core/format";
import { clipInput, opt, type CompileInputs, type PortSpec } from "#core/nodes/define";
import type { ClipValue, CompileValue, MediaValue } from "#core/schema/values";
import { PORT_WORDS, type PortType } from "#core/schema/workflow";

type Conversion = (value: CompileValue) => CompileValue | undefined;

const imageAsMedia: Conversion = (value): MediaValue | undefined =>
  value.type === "image"
    ? { type: "media", kind: "image", src: value.src, width: value.width, height: value.height }
    : undefined;

const recordingAsMedia: Conversion = (value): MediaValue | undefined =>
  value.type === "recording"
    ? {
        type: "media",
        kind: "video",
        src: value.src,
        durationMs: value.durationMs,
        width: value.viewport.width,
        height: value.viewport.height,
        hasAudio: false,
      }
    : undefined;

const CONVERT: { [T in PortType]?: Partial<Record<PortType, Conversion>> } = {
  media: { image: imageAsMedia, recording: recordingAsMedia },
};

export function canConnect(source: PortType, target: PortType): boolean {
  return source === target || !!CONVERT[target]?.[source];
}

export function convertInput(
  value: CompileValue | undefined,
  spec: Pick<PortSpec, "type"> | undefined,
): CompileValue | undefined {
  if (!value || !spec || value.type === spec.type) return value;
  return CONVERT[spec.type]?.[value.type]?.(value);
}

export function portLabel(spec: { type: PortType; label?: string }): string {
  return spec.label ?? humanize(PORT_WORDS[spec.type]);
}

export function portLabels(ports: Record<string, PortSpec>): string[] {
  return Object.values(ports).map(portLabel);
}

export const SESSION_PORT = "session";

export const IN_PORTS = ["in1", "in2", "in3", "in4", "in5", "in6"] as const;

type InPort = (typeof IN_PORTS)[number];

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
