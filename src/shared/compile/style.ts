import type { z } from "zod";
import type { NodeDef } from "../nodes/define";
import type { ClipValue, StyleEntry } from "../schema/values";
import type { RecordingProps } from "./recording";

type Retiming = { nodeId: string; fps: number };

type StyleStep<P> = {
  restyle?: (props: RecordingProps, params: P, nodeId: string) => RecordingProps;
  retime?: (clip: ClipValue, params: P, at: Retiming) => ClipValue;
};

type EntryStep = {
  restyle: (props: RecordingProps, entry: StyleEntry) => RecordingProps;
  retime: (clip: ClipValue, entry: StyleEntry, fps: number) => ClipValue;
};

export type StyleSteps = ReadonlyArray<readonly [string, EntryStep]>;

export function styleStep<S extends z.ZodObject>(
  def: NodeDef<S>,
  { restyle, retime }: StyleStep<z.infer<S>>,
): readonly [string, EntryStep] {
  const parse = (entry: StyleEntry) => def.params.parse(entry.params);
  return [
    def.type,
    {
      restyle: (props, entry) => (restyle ? restyle(props, parse(entry), entry.nodeId) : props),
      retime: (clip, entry, fps) =>
        retime ? retime(clip, parse(entry), { nodeId: entry.nodeId, fps }) : clip,
    },
  ];
}

export type StyleTable = ReadonlyMap<string, EntryStep>;

export const restyleWith = (
  table: StyleTable,
  props: RecordingProps,
  entries: readonly StyleEntry[],
): RecordingProps => entries.reduce((p, e) => table.get(e.type)?.restyle(p, e) ?? p, props);

export const retimeWith = (
  table: StyleTable,
  clip: ClipValue,
  entries: readonly StyleEntry[],
  fps: number,
): ClipValue => entries.reduce((c, e) => table.get(e.type)?.retime(c, e, fps) ?? c, clip);
