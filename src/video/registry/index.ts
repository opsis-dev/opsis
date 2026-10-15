import type { TransitionPresentation } from "@remotion/transitions";
import type React from "react";
import { useEffect, useState } from "react";
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
import type { ElementImpl } from "./types";

// oxlint-disable-next-line typescript/no-explicit-any
type AnyComponent = React.ComponentType<any>;
type Presentation = TransitionPresentation<Record<string, unknown>>;
type Size = { width: number; height: number };

const core: Record<string, AnyComponent> = {
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

const loaded = new Map<string, ElementImpl>();
const loading = new Map<string, Promise<ElementImpl | undefined>>();

const isCore = (key: string) => key in core || key in coreTransitions;

export function loadElement(key: string): Promise<ElementImpl | undefined> {
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
  const [, rerender] = useState(0);
  const signature = missing.join("|");
  useEffect(() => {
    if (!signature) return;
    const handle = delayRender(`Loading ${signature}`);
    let alive = true;
    Promise.all(signature.split("|").map(loadElement))
      .then(() => {
        if (alive) rerender((n) => n + 1);
        continueRender(handle);
      })
      .catch((error: unknown) => cancelRender(error));
    return () => {
      alive = false;
    };
  }, [signature]);
  return missing.length === 0;
}

export function getComponent(key: string): AnyComponent | undefined {
  return core[key] ?? (loaded.get(key) as AnyComponent | undefined);
}

export function getPresentation(
  key: string,
  params: Record<string, unknown>,
  size: Size,
): Presentation | null {
  const builtIn = coreTransitions[key];
  if (builtIn) return builtIn(params, size);
  const impl = loaded.get(key) as ((p: Record<string, unknown>) => Presentation) | undefined;
  return impl ? impl(params) : null;
}

export const isKnownElement = (key: string) => isCore(key) || key in generatedElementLoaders;
