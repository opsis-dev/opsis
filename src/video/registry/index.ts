import type React from "react";
import type { CoreElementProps } from "../../shared/elements/core";
import { isKeyOf } from "../../shared/guards";
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
  "core/placeholder": { title: string; subtitle: string };
  "core/still": { src: string; look: StillLook; backdrop: string; shot?: Shot };
  "core/captions": { captions: CaptionsValue["captions"] };
  "core/mosaic": { children?: React.ReactNode };
};

const LAZY_CORE_ELEMENTS = {
  "core/logo-3d": () => import("../three/logo-3d").then((m) => component(m.Logo3D)),
  "core/card-3d": () => import("../three/card-3d").then((m) => component(m.Card3D)),
  "core/shapes-3d": () => import("../three/shapes-3d").then((m) => component(m.Shapes3D)),
  "core/text-3d": () => import("../three/text-3d").then((m) => component(m.Text3D)),
  "core/model-3d": () => import("../three/model-3d").then((m) => component(m.Model3D)),
  "core/screen-3d": () => import("../three/screen-3d").then((m) => component(m.Screen3D)),
} satisfies Record<string, () => Promise<ElementImpl>>;

type EagerCoreKey = Exclude<keyof CoreProps, keyof typeof LAZY_CORE_ELEMENTS>;

const CORE_ELEMENTS: { [K in EagerCoreKey]: React.FC<CoreProps[K]> } = {
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

const lazyLoaderOf = (key: string): (() => Promise<ElementImpl>) | undefined => {
  if (Object.hasOwn(generatedElementLoaders, key)) return generatedElementLoaders[key];
  return isKeyOf(LAZY_CORE_ELEMENTS, key) ? LAZY_CORE_ELEMENTS[key] : undefined;
};

const isLazy = (key: string) => lazyLoaderOf(key) !== undefined;

const isCore = (key: string) => core.has(key) || Object.hasOwn(coreTransitions, key);

function loadElement(key: string): Promise<ElementImpl | undefined> {
  const hit = loaded.get(key);
  if (hit) return Promise.resolve(hit);
  const loader = lazyLoaderOf(key);
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

const needsLoad = (key: string | undefined): key is string =>
  !!key && !isCore(key) && !loaded.has(key) && isLazy(key);

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

const isKnownElement = (key: string) => isCore(key) || isLazy(key);

type ResolvedElement<T> =
  | { status: "unknown" }
  | { status: "loading" }
  | { status: "ready"; impl: T };

export function useResolvedElement<T>(
  key: string | undefined,
  get: (key: string) => T | undefined,
): ResolvedElement<T> | null {
  useElement(key);
  if (key === undefined) return null;
  if (!isKnownElement(key)) return { status: "unknown" };
  const impl = get(key);
  return impl ? { status: "ready", impl } : { status: "loading" };
}
