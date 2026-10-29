import type { Artifact, CompileValue, NodeResult } from "@/generated/bindings";

export const artifactName = (artifact: Artifact): string => artifact.label ?? artifact.path;

type ValueOf<K extends CompileValue["type"]> = Extract<CompileValue, { type: K }>;

const isValue = <K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): value is ValueOf<K> => value?.type === type;

export const asValue = <K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): ValueOf<K> | undefined => (isValue(value, type) ? value : undefined);

export const outputOf = <K extends CompileValue["type"]>(
  result: NodeResult | undefined,
  port: string,
  type: K,
): ValueOf<K> | undefined => asValue(result?.outputs[port], type);
