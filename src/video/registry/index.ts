import { useEffect, useSyncExternalStore } from "react";
import { cancelRender, continueRender, delayRender } from "remotion";
import {
  Backdrop,
  Gradient,
  LightLeakElement,
  NoiseField,
  Paper,
  Solid,
  Starburst,
} from "../core/backgrounds";
import { LottieElement } from "../core/lottie";
import { ImageElement, StillElement, VideoElement, VideoLoop } from "../core/media";
import { Mosaic } from "../core/mosaic";
import { Placeholder } from "../core/placeholder";
import { RecordingClip } from "../core/recording-clip";
import type { Size } from "../core/still-layout";
import { Callout, Captions, KineticHeadline, LowerThird, Title } from "../core/text";
import { coreTransitions } from "../core/transitions";
import { generatedElementLoaders } from "./components.generated";
import { component } from "./framing";
import {
  isPresentation,
  type ElementComponent,
  type ElementImpl,
  type Presentation,
} from "./types";

const core: Partial<Record<string, ElementComponent>> = {
  "core/recording": component(RecordingClip),
  "core/placeholder": component(Placeholder),
  "core/still": component(StillElement),
  "core/title": component(Title),
  "core/kinetic-headline": component(KineticHeadline),
  "core/lower-third": component(LowerThird),
  "core/callout": component(Callout),
  "core/captions": component(Captions),
  "core/image": component(ImageElement),
  "core/lottie": component(LottieElement),
  "core/mosaic": component(Mosaic),
  "core/video": component(VideoElement),
  "core/video-loop": component(VideoLoop),
  "core/solid": component(Solid),
  "core/gradient": component(Gradient),
  "core/noise": component(NoiseField),
  "core/starburst": component(Starburst),
  "core/paper": component(Paper),
  "core/backdrop": component(Backdrop),
  "core/light-leak": component(LightLeakElement),
};

const loaded = new Map<string, ElementImpl>();
const loading = new Map<string, Promise<ElementImpl | undefined>>();
const loadListeners = new Set<() => void>();

function subscribeToLoads(listener: () => void): () => void {
  loadListeners.add(listener);
  return () => loadListeners.delete(listener);
}

const isCore = (key: string) => key in core || key in coreTransitions;

function loadElement(key: string): Promise<ElementImpl | undefined> {
  const hit = loaded.get(key);
  if (hit) return Promise.resolve(hit);
  const loader = generatedElementLoaders[key];
  if (!loader) return Promise.resolve(undefined);
  let pending = loading.get(key);
  if (!pending) {
    pending = loader().then((impl) => {
      loaded.set(key, impl);
      for (const listener of loadListeners) listener();
      return impl;
    });
    loading.set(key, pending);
  }
  return pending;
}

const KEY_SEPARATOR = "\n";

const missingKeys = (keys: Array<string | undefined>): string =>
  keys
    .filter((k): k is string => !!k && !isCore(k) && !loaded.has(k) && k in generatedElementLoaders)
    .join(KEY_SEPARATOR);

export function useElements(keys: Array<string | undefined>): boolean {
  const snapshot = () => missingKeys(keys);
  const missing = useSyncExternalStore(subscribeToLoads, snapshot, snapshot);
  useEffect(() => {
    if (!missing) return;
    const wanted = missing.split(KEY_SEPARATOR);
    const handle = delayRender(`Loading ${wanted.join(", ")}`);
    Promise.all(wanted.map(loadElement))
      .then(() => continueRender(handle))
      .catch((error: unknown) => cancelRender(error));
  }, [missing]);
  return missing === "";
}

export function getComponent(key: string): ElementComponent | undefined {
  const impl = core[key] ?? loaded.get(key);
  return impl && !isPresentation(impl) ? impl : undefined;
}

export function getPresentation(
  key: string,
  params: Record<string, unknown>,
  size: Size,
): Presentation | null {
  const impl = coreTransitions[key] ?? loaded.get(key);
  return impl && isPresentation(impl) ? impl(params, size) : null;
}

export const isKnownElement = (key: string) => isCore(key) || key in generatedElementLoaders;
