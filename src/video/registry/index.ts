import type React from "react";
import type { RecordingProps } from "../../shared/compile/recording";
import type { CoreElementProps } from "../../shared/elements/core";
import type { CaptionsValue } from "../../shared/schema/values";
import {
  Backdrop,
  Gradient,
  LightLeakElement,
  NoiseField,
  Paper,
  Solid,
  Starburst,
} from "../core/backgrounds";
import { Callout } from "../core/callout";
import { Captions } from "../core/captions";
import { KineticHeadline } from "../core/kinetic-headline";
import { useHeldLoad } from "../core/loading";
import { LottieElement } from "../core/lottie";
import { LowerThird } from "../core/lower-third";
import { ImageElement, StillElement, VideoElement, VideoLoop } from "../core/media";
import { Mosaic } from "../core/mosaic";
import { Placeholder } from "../core/placeholder";
import { RecordingClip } from "../core/recording-clip";
import type { Shot, Size, StillLook } from "../core/still-layout";
import { Title } from "../core/title";
import { coreTransitions } from "../core/transitions";
import { generatedElementLoaders } from "./components.generated";
import { component } from "./framing";
import {
  isPresentation,
  isSequencer,
  type ElementComponent,
  type ElementImpl,
  type Presentation,
  type SceneSequencer,
} from "./types";

type CoreProps = CoreElementProps & {
  "core/recording": RecordingProps;
  "core/placeholder": { title: string; subtitle: string };
  "core/still": { src: string; look: StillLook; backdrop: string; shot?: Shot };
  "core/captions": { captions: CaptionsValue["captions"] };
  "core/mosaic": { children?: React.ReactNode };
};

const CORE_ELEMENTS: { [K in keyof CoreProps]: React.FC<CoreProps[K]> } = {
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
  "core/backdrop": Backdrop,
  "core/light-leak": LightLeakElement,
};

const core = new Map(
  Object.entries(CORE_ELEMENTS).map(([key, element]) => [key, component(element)]),
);

const loaded = new Map<string, ElementImpl>();
const loading = new Map<string, Promise<ElementImpl | undefined>>();

const isGenerated = (key: string) => Object.hasOwn(generatedElementLoaders, key);

const isCore = (key: string) => core.has(key) || Object.hasOwn(coreTransitions, key);

function loadElement(key: string): Promise<ElementImpl | undefined> {
  const hit = loaded.get(key);
  if (hit) return Promise.resolve(hit);
  if (!isGenerated(key)) return Promise.resolve(undefined);
  let pending = loading.get(key);
  if (!pending) {
    pending = generatedElementLoaders[key]().then((impl) => {
      loaded.set(key, impl);
      return impl;
    });
    loading.set(key, pending);
  }
  return pending;
}

const needsLoad = (key: string | undefined): key is string =>
  !!key && !isCore(key) && !loaded.has(key) && isGenerated(key);

export function useElement(key: string | undefined): void {
  useHeldLoad(`Loading ${key}`, needsLoad(key) ? key : null, loadElement);
}

export function getComponent(key: string): ElementComponent | undefined {
  const impl = core.get(key) ?? loaded.get(key);
  return impl && !isPresentation(impl) && !isSequencer(impl) ? impl : undefined;
}

export function getSequencer(key: string): SceneSequencer | undefined {
  const impl = loaded.get(key);
  return impl && isSequencer(impl) ? impl : undefined;
}

export function getPresentation(
  key: string,
  params: Record<string, unknown>,
  size: Size,
): Presentation | null {
  const impl =
    (Object.hasOwn(coreTransitions, key) ? coreTransitions[key] : undefined) ?? loaded.get(key);
  return impl && isPresentation(impl) ? impl.create(params, size) : null;
}

export const isKnownElement = (key: string) => isCore(key) || isGenerated(key);
