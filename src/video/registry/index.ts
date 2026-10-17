import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
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
import { component } from "./framing";
import type { ElementComponent, ElementImpl, PresentationFactory } from "./types";

type Presentation = TransitionPresentation<Record<string, unknown>>;
type Size = { width: number; height: number };

const CORE_COMPONENTS: Record<string, React.ComponentType<never>> = {
  "core/recording": RecordingClip,
  "core/placeholder": Placeholder,
  "core/still": StillElement,
  "core/title": Title,
  "core/kinetic-headline": KineticHeadline,
  "core/lower-third": LowerThird,
  "core/callout": Callout,
  "core/captions": Captions,
  "core/image": ImageElement,
  "core/lottie": LottieElement,
  "core/mosaic": Mosaic,
  "core/video": VideoElement,
  "core/video-loop": VideoLoop,
  "core/solid": Solid,
  "core/gradient": Gradient,
  "core/noise": NoiseField,
  "core/starburst": Starburst,
  "core/paper": Paper,
  "core/backdrop": StudioBackdrop,
  "core/light-leak": LightLeakElement,
};

const core: Partial<Record<string, ElementComponent>> = Object.fromEntries(
  Object.entries(CORE_COMPONENTS).map(([key, element]) => [key, component(element)]),
);

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
  const signature = JSON.stringify(missing);
  useEffect(() => {
    const wanted: string[] = JSON.parse(signature);
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
  }, [signature]);
  return missing.length === 0;
}

export function getComponent(key: string): ElementComponent | undefined {
  return core[key] ?? (loaded.get(key) as ElementComponent | undefined);
}

export function getPresentation(
  key: string,
  params: Record<string, unknown>,
  size: Size,
): Presentation | null {
  const builtIn = coreTransitions[key];
  if (builtIn) return builtIn(params, size);
  const impl = loaded.get(key) as PresentationFactory | undefined;
  return impl ? impl(params) : null;
}

export const isKnownElement = (key: string) => isCore(key) || key in generatedElementLoaders;
