import { useEffect, useReducer } from "react";
import { cancelRender, continueRender, delayRender } from "remotion";
import {
  Gradient,
  LightLeakElement,
  NoiseField,
  Paper,
  Solid,
  Starburst,
  StudioBackdrop,
} from "../core/backgrounds";
import { LottieElement } from "../core/lottie";
import { ImageElement, StillElement, VideoElement, VideoLoop } from "../core/media";
import { Mosaic } from "../core/mosaic";
import { Placeholder } from "../core/placeholder";
import { RecordingClip } from "../core/recording-clip";
import { Callout, Captions, KineticHeadline, LowerThird, Title } from "../core/text";
import { coreTransitions } from "../core/transitions";
import { generatedElementLoaders } from "./components.generated";
import { useStableValue } from "../core/loading";
import type { Size } from "../core/still-layout";
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
  "core/backdrop": component(StudioBackdrop),
  "core/light-leak": component(LightLeakElement),
};

const loaded = new Map<string, ElementImpl>();
const loading = new Map<string, Promise<ElementImpl | undefined>>();

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
      return impl;
    });
    loading.set(key, pending);
  }
  return pending;
}

export function useElements(keys: Array<string | undefined>): boolean {
  const missing = keys.filter(
    (k): k is string => !!k && !isCore(k) && !loaded.has(k) && k in generatedElementLoaders,
  );
  const [, refresh] = useReducer((n: number) => n + 1, 0);
  const wanted = useStableValue(missing);
  useEffect(() => {
    if (wanted.length === 0) return;
    const handle = delayRender(`Loading ${wanted.join(", ")}`);
    let alive = true;
    Promise.all(wanted.map(loadElement))
      .then(() => {
        if (alive) refresh();
        continueRender(handle);
      })
      .catch((error: unknown) => cancelRender(error));
    return () => {
      alive = false;
    };
  }, [wanted]);
  return missing.length === 0;
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
