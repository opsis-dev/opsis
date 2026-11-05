import type { Artifact, CompileValue, NodeResult } from "@/generated/bindings";

export function artifactName(artifact: Artifact): string {
  return artifact.label ?? artifact.path;
}

type ValueOf<K extends CompileValue["type"]> = Extract<CompileValue, { type: K }>;

function isValue<K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): value is ValueOf<K> {
  return value?.type === type;
}

function asValue<K extends CompileValue["type"]>(
  value: CompileValue | undefined,
  type: K,
): ValueOf<K> | undefined {
  return isValue(value, type) ? value : undefined;
}

export function outputOf<K extends CompileValue["type"]>(
  result: NodeResult | undefined,
  port: string,
  type: K,
): ValueOf<K> | undefined {
  return asValue(result?.outputs[port], type);
}
