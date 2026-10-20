import { Audio } from "@remotion/media";
import React, { createElement, useMemo } from "react";
import {
  AbsoluteFill,
  Freeze,
  interpolate,
  Loop,
  Sequence,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import { groupOf, type GroupProps, type SequencerProps } from "../shared/compile/clip";
import { remapFrame } from "../shared/compile/remap";
import { ERROR_COLOR, STAGE_COLOR } from "../shared/elements/colors";
import { CODE_FONT, MAIN_FONT } from "../shared/fonts";
import { percent } from "../shared/format";
import { MIN_CROP } from "../shared/limits";
import { clamp01, EPSILON } from "../shared/math";
import { duckGain, duckingSpans, type Span } from "../shared/mix/duck";
import type { Rect } from "../shared/schema/actions";
import type {
  AudioItem,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
  VisualEnvelope,
} from "../shared/schema/timeline";
import { envelopeAt } from "../shared/timeline/envelope";
import { flattenSpec, type FlatTimeline } from "../shared/timeline/flatten";
import {
  AssetProvider,
  DEFAULT_QUALITY,
  DEFAULT_REFRAME,
  QualityProvider,
  ReframeProvider,
  resolveAssetRef,
  resolveProps,
  useAssetBase,
} from "./assets";
import { CLAMP_RIGHT, fadeWindow } from "./core/anim";
import type { Size } from "./core/still-layout";
import { FontProvider } from "./fonts/font-provider";
import {
  getComponent,
  getPresentation,
  getSequencer,
  useElement,
  useResolvedElement,
} from "./registry";
import { PaintRule } from "./registry/framing";

const ELEMENT_FONTS: React.CSSProperties &
  Record<"--font-geist-sans" | "--font-geist-mono", string> = {
  "--font-geist-sans": MAIN_FONT,
  "--font-geist-mono": CODE_FONT,
};

const PREMOUNT_FRAMES = 30;
const PREMOUNTED = new Set(["core/group", "core/recording"]);

export const DemoTimeline: React.FC<TimelineSpec> = (spec) => (
  <AssetProvider value={spec.assets ?? {}}>
    <ReframeProvider value={spec.reframe ?? DEFAULT_REFRAME}>
      <QualityProvider value={spec.quality ?? DEFAULT_QUALITY}>
        <AbsoluteFill
          style={{
            ...ELEMENT_FONTS,
            background: spec.background ?? STAGE_COLOR,
            overflow: "hidden",
          }}
        >
          <PaintRule />
          <FontProvider fonts={spec.fonts} tracks={spec.tracks}>
            <Tracks tracks={spec.tracks} />
          </FontProvider>
          {spec.mix ? <MasterStem src={spec.mix.src} /> : <AudioLayers spec={spec} />}
        </AbsoluteFill>
      </QualityProvider>
    </ReframeProvider>
  </AssetProvider>
);

const Tracks: React.FC<{ tracks: TimelineTrack[] }> = ({ tracks }) => (
  <>
    {tracks.map((track) => (
      <AbsoluteFill key={track.id}>
        {track.items.map((item) => (
          <Sequence
            key={item.id}
            from={item.from}
            durationInFrames={item.durationInFrames}
            name={item.component}
            premountFor={PREMOUNTED.has(item.component) ? PREMOUNT_FRAMES : undefined}
          >
            <TransitionWrap item={item}>
              {item.envelope ? (
                <EnvelopeWrap envelope={item.envelope}>
                  <ItemContent item={item} />
                </EnvelopeWrap>
              ) : (
                <ItemContent item={item} />
              )}
            </TransitionWrap>
          </Sequence>
        ))}
      </AbsoluteFill>
    ))}
  </>
);

const EnvelopeWrap: React.FC<{ envelope: VisualEnvelope; children: React.ReactNode }> = ({
  envelope,
  children,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const opacity = envelopeAt(envelope.opacity, frame, 1);
  const x = envelopeAt(envelope.x, frame, 0) * width;
  const y = envelopeAt(envelope.y, frame, 0) * height;
  const scale = envelopeAt(envelope.scale, frame, 1);
  return (
    <AbsoluteFill
      style={{
        opacity: clamp01(opacity),
        transform: `translate(${x}px, ${y}px) scale(${Math.max(0, scale)})`,
      }}
    >
      {children}
    </AbsoluteFill>
  );
};

const MasterStem: React.FC<{ src: string }> = ({ src }) => {
  const base = useAssetBase();
  return <Audio src={resolveAssetRef(src, base)} />;
};

const ItemContent: React.FC<{ item: TimelineItem }> = ({ item }) => {
  const group = groupOf(item);
  return group ? <Group {...group} /> : <ElementItem item={item} />;
};

const ElementItem: React.FC<{ item: TimelineItem }> = ({ item }) => {
  const base = useAssetBase();
  const element = useResolvedElement(item.component, getComponent);
  if (element?.status === "unknown") return <MissingElement name={item.component} />;
  return element?.status === "ready" ? (
    <DesignBox design={item.design} fill>
      {createElement(element.impl, resolveProps(item.props, base))}
    </DesignBox>
  ) : null;
};

const cropScale = (crop: Rect): number => 1 / Math.max(MIN_CROP, crop.w, crop.h);

const FILL: React.CSSProperties = { position: "absolute", inset: 0 };

const SceneSequence: React.FC<{ sequencer: SequencerProps; tracks: TimelineTrack[] }> = ({
  sequencer,
  tracks,
}) => {
  const base = useAssetBase();
  const element = useResolvedElement(sequencer.component, getSequencer);
  if (element?.status === "unknown") return <MissingElement name={sequencer.component} />;
  if (element?.status !== "ready") return null;
  const { Scenes, Item } = element.impl;
  const scenes = tracks
    .flatMap((track) => track.items)
    .map((scene, i) => ({
      durationInFrames: scene.durationInFrames,
      bg: sequencer.backgrounds[i] ?? undefined,
      content: (
        <Sequence from={scene.from} layout="none">
          {(groupOf(scene)?.tracks ?? []).map((track, index) => (
            <Item key={track.id} index={index} style={FILL}>
              <Tracks tracks={[track]} />
            </Item>
          ))}
        </Sequence>
      ),
    }));
  return <Scenes scenes={scenes} config={resolveProps(sequencer.props, base)} />;
};

const Group: React.FC<GroupProps> = ({
  tracks,
  startFrom,
  remap,
  loopEvery,
  crop,
  wrapper,
  background,
  design,
  sequencer,
}) => {
  const frame = useCurrentFrame();
  let layers: React.ReactNode = sequencer ? (
    <SceneSequence sequencer={sequencer} tracks={tracks} />
  ) : (
    <Tracks tracks={tracks} />
  );
  if (startFrom)
    layers = (
      <Sequence from={-startFrom} layout="none">
        {layers}
      </Sequence>
    );
  if (loopEvery !== undefined && loopEvery > 0)
    layers = <Loop durationInFrames={loopEvery}>{layers}</Loop>;
  if (remap?.length)
    layers = (
      <Freeze frame={Math.max(0, Math.floor(remapFrame(remap, frame) + EPSILON))}>{layers}</Freeze>
    );
  if (crop) layers = <CropBox crop={crop}>{layers}</CropBox>;
  return (
    <AbsoluteFill style={background ? { background } : undefined}>
      <DesignBox design={design}>
        <Wrapped wrapper={wrapper}>{layers}</Wrapped>
      </DesignBox>
    </AbsoluteFill>
  );
};

type Wrap<P> = React.FC<P & { children: React.ReactNode }>;

const CropBox: Wrap<{ crop: Rect }> = ({ crop, children }) => {
  const scale = cropScale(crop);
  const w = crop.w * scale;
  const h = crop.h * scale;
  return (
    <AbsoluteFill>
      <div
        style={{
          position: "absolute",
          overflow: "hidden",
          left: percent((1 - w) / 2),
          top: percent((1 - h) / 2),
          width: percent(w),
          height: percent(h),
        }}
      >
        <div
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: percent(1 / w),
            height: percent(1 / h),
            transformOrigin: "0 0",
            transform: `scale(${scale}) translate(${percent(-crop.x)}, ${percent(-crop.y)})`,
          }}
        >
          {children}
        </div>
      </div>
    </AbsoluteFill>
  );
};

const Wrapped: Wrap<{ wrapper?: GroupProps["wrapper"] }> = ({ wrapper, children }) => {
  const base = useAssetBase();
  const element = useResolvedElement(wrapper?.component, getComponent);
  if (!wrapper || element?.status === "loading") return <>{children}</>;
  if (element?.status !== "ready")
    return (
      <>
        {children}
        <MissingElement name={wrapper.component} />
      </>
    );
  return createElement(
    element.impl,
    resolveProps(wrapper.props, base),
    <AbsoluteFill>{children}</AbsoluteFill>,
  );
};

const DesignBox: Wrap<{ design?: Size; fill?: boolean }> = ({ design, fill, children }) => {
  const { width, height } = useVideoConfig();
  if (!design || (width === design.width && height === design.height)) return <>{children}</>;
  const scale = Math.min(width / design.width, height / design.height);
  const box = fill
    ? { width: Math.ceil(width / scale), height: Math.ceil(height / scale) }
    : design;
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", overflow: "hidden" }}>
      <div
        style={{
          position: "relative",
          flexShrink: 0,
          width: box.width,
          height: box.height,
          transform: `scale(${scale})`,
        }}
      >
        <Sequence layout="none" width={box.width} height={box.height}>
          {children}
        </Sequence>
      </div>
    </AbsoluteFill>
  );
};

const noop = () => undefined;

const TransitionWrap: React.FC<{ item: TimelineItem; children: React.ReactNode }> = ({
  item,
  children,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  useElement(item.enter?.kind);
  useElement(item.exit?.kind);
  const active = activeTransition(item, frame);
  if (!active) return <>{children}</>;
  const presentation = getPresentation(active.ref.kind, active.ref.params ?? {}, { width, height });
  if (!presentation) return <>{children}</>;
  const Presentation = presentation.component;
  return (
    <Presentation
      presentationProgress={active.progress}
      presentationDirection={active.direction}
      passedProps={presentation.props}
      presentationDurationInFrames={active.ref.durationInFrames}
      onElementImage={noop}
      onUnmount={noop}
      bothEnteringAndExiting={false}
    >
      {children}
    </Presentation>
  );
};

type ActiveTransition = {
  ref: TransitionRef;
  direction: "entering" | "exiting";
  progress: number;
};

function activeTransition(item: TimelineItem, frame: number): ActiveTransition | null {
  const { enter, exit, durationInFrames } = item;
  const progressFrom = (ref: TransitionRef, start: number) =>
    interpolate(frame, [start, start + ref.durationInFrames], [0, 1], CLAMP_RIGHT);
  if (enter && frame < enter.durationInFrames)
    return { ref: enter, direction: "entering", progress: progressFrom(enter, 0) };
  const exitStart = exit ? durationInFrames - exit.durationInFrames : 0;
  if (exit && frame >= exitStart)
    return { ref: exit, direction: "exiting", progress: progressFrom(exit, exitStart) };
  return null;
}

const NO_SPANS: Span[] = [];

const duckKey = (keys: string[]) => keys.join("|");

const AudioLayers: React.FC<{ spec: TimelineSpec }> = ({ spec }) => {
  const layers = useMemo(() => {
    const spansByKey = new Map<string, Span[]>();
    let flat: FlatTimeline | null = null;
    const spansFor = (keys: string[]) => {
      const key = duckKey(keys);
      const known = spansByKey.get(key);
      if (known) return known;
      flat ??= flattenSpec(spec);
      const spans = duckingSpans(spec, new Set(keys), flat);
      spansByKey.set(key, spans);
      return spans;
    };
    return spec.audio
      .filter((a) => a.gain !== 0)
      .map((item) => ({ item, spans: item.duck ? spansFor(item.duck.keys) : NO_SPANS }));
  }, [spec]);
  return (
    <>
      {layers.map(({ item, spans }) => (
        <AudioLayer key={item.id} item={item} spans={spans} />
      ))}
    </>
  );
};

function audioGainAt(item: AudioItem, f: number, duration: number, spans: Span[]): number {
  const baseVolume = (item.volume ?? 1) * (item.gain ?? 1);
  const fade = fadeWindow(f, 0, duration, {
    in: item.fadeInFrames ?? 0,
    out: item.fadeOutFrames ?? 0,
  });
  const duck = item.duck
    ? duckGain(item.from + f, spans, item.duck.level, item.duck.fadeFrames)
    : 1;
  const env = envelopeAt(item.envelope?.volume, f, 1);
  return Math.max(0, baseVolume * fade * duck * env);
}

const AudioLayer: React.FC<{ item: AudioItem; spans: Span[] }> = ({ item, spans }) => {
  const base = useAssetBase();
  const { durationInFrames: total } = useVideoConfig();
  const duration = item.durationInFrames ?? Math.max(1, total - item.from);
  return (
    <Sequence from={item.from} durationInFrames={duration} name={`audio ${item.id}`} layout="none">
      <Audio
        src={resolveAssetRef(item.src, base)}
        loop={item.loop}
        playbackRate={item.rate}
        trimBefore={item.trimBefore}
        volume={(f) => audioGainAt(item, f, duration, spans)}
      />
    </Sequence>
  );
};

const MissingElement: React.FC<{ name: string }> = ({ name }) => (
  <AbsoluteFill
    style={{
      alignItems: "center",
      justifyContent: "center",
      color: ERROR_COLOR,
      fontFamily: MAIN_FONT,
      fontSize: 40,
    }}
  >
    Unknown element: {name}
  </AbsoluteFill>
);
