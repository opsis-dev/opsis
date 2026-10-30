import type { Size } from "#video/core/still-layout.ts";
import { ELEMENTS } from "#video/elements/index.ts";
import {
  isPresentation,
  isSequencer,
  type ElementComponent,
  type ElementImpl,
  type Presentation,
  type SceneSequencer,
} from "#video/registry/types.ts";

function implOf(key: string): ElementImpl | undefined {
  return Object.hasOwn(ELEMENTS, key) ? ELEMENTS[key] : undefined;
}

export function getComponent(key: string): ElementComponent | undefined {
  const impl = implOf(key);
  return impl && !isPresentation(impl) && !isSequencer(impl) ? impl : undefined;
}

export function getSequencer(key: string): SceneSequencer | undefined {
  const impl = implOf(key);
  return impl && isSequencer(impl) ? impl : undefined;
}

export function getPresentation(
  key: string,
  params: Record<string, unknown>,
  size: Size,
): Presentation | null {
  const impl = implOf(key);
  return impl && isPresentation(impl) ? impl.create(params, size) : null;
}

type ResolvedElement<T> = { status: "unknown" } | { status: "ready"; impl: T };

export function resolveElement<T>(
  key: string | undefined,
  get: (key: string) => T | undefined,
): ResolvedElement<T> | null {
  if (key === undefined) return null;
  const impl = get(key);
  return impl ? { status: "ready", impl } : { status: "unknown" };
}
