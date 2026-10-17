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
import { Audio } from "@remotion/media";
import { groupOf, type GroupProps } from "../shared/compile/clip";
import { remapFrame } from "../shared/compile/remap";
import { duckGain, duckingSpans, type Span } from "../shared/compile/duck";
import { envelopeAt } from "../shared/timeline/envelope";
import type {
  AudioItem,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
  VisualEnvelope,
} from "../shared/schema/timeline";
import {
  AssetProvider,
  QualityProvider,
  ReframeProvider,
  resolveAssetRef,
  resolveProps,
  useAssetBase,
} from "./assets";
import { FontProvider } from "./fonts/font-provider";
import { getComponent, getPresentation, isKnownElement, useElements } from "./registry";
import { clamp01 } from "../shared/math";
import { MAIN_FONT } from "../shared/fonts";
import { percent } from "../shared/format";
import { CLAMP_RIGHT, fadeWindow } from "./core/anim";
import { sameShape } from "./core/frame";
import type { Size } from "./core/still-layout";

const PREMOUNT_FRAMES = 30;
const PREMOUNTED = new Set(["core/group", "core/recording"]);
const MIN_CROP = 0.05;

export const DemoTimeline: React.FC<TimelineSpec> = (spec) => (
  <AssetProvider value={spec.assets ?? {}}>
    <ReframeProvider value={spec.reframe ?? "follow"}>
      <QualityProvider value={spec.quality ?? "exact"}>
        <AbsoluteFill style={{ background: spec.background ?? "#000", overflow: "hidden" }}>
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
              <Enveloped envelope={item.envelope}>
                <ItemContent item={item} />
              </Enveloped>
            </TransitionWrap>
          </Sequence>
        ))}
      </AbsoluteFill>
    ))}
  </>
);

const Enveloped: React.FC<{ envelope?: VisualEnvelope; children: React.ReactNode }> = ({
  envelope,
  children,
}) => (envelope ? <EnvelopeWrap envelope={envelope}>{children}</EnvelopeWrap> : <>{children}</>);

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
  const ready = useElements([item.component]);
  if (!isKnownElement(item.component)) return <MissingElement name={item.component} />;
  const component = ready ? getComponent(item.component) : undefined;
  return component ? createElement(component, resolveProps(item.props, base)) : null;
};

type Wrap = (inner: React.ReactNode) => React.ReactNode;

const Group: React.FC<GroupProps> = ({
  tracks,
  startFrom,
  remap,
  loopEvery,
  crop,
  wrapper,
  background,
  design,
}) => {
  const frame = useCurrentFrame();
  const base = useAssetBase();
  useElements([wrapper?.component]);
  const Wrapper = wrapper ? getComponent(wrapper.component) : undefined;
  const wraps: Array<Wrap | null> = [
    startFrom
      ? (inner) => (
          <Sequence from={-startFrom} layout="none">
            {inner}
          </Sequence>
        )
      : null,
    loopEvery && loopEvery > 0
      ? (inner) => <Loop durationInFrames={loopEvery}>{inner}</Loop>
      : null,
    remap?.length
      ? (inner) => (
          <Freeze frame={Math.max(0, Math.round(remapFrame(remap, frame)))}>{inner}</Freeze>
        )
      : null,
    crop
      ? (inner) => {
          const s = 1 / Math.max(MIN_CROP, crop.w, crop.h);
          return (
            <AbsoluteFill style={{ overflow: "hidden" }}>
              <AbsoluteFill
                style={{
                  transformOrigin: "0 0",
                  transform: `scale(${s}) translate(${percent(-crop.x)}, ${percent(-crop.y)})`,
                }}
              >
                {inner}
              </AbsoluteFill>
            </AbsoluteFill>
          );
        }
      : null,
    wrapper && Wrapper
      ? (inner) =>
          createElement(
            Wrapper,
            resolveProps(wrapper.props, base),
            <AbsoluteFill>{inner}</AbsoluteFill>,
          )
      : null,
    design ? (inner) => <DesignBox design={design}>{inner}</DesignBox> : null,
  ];
  const content = wraps.reduce<React.ReactNode>(
    (inner, wrap) => (wrap ? wrap(inner) : inner),
    <Tracks tracks={tracks} />,
  );
  return <AbsoluteFill style={background ? { background } : undefined}>{content}</AbsoluteFill>;
};

const DesignBox: React.FC<{
  design: Size;
  children: React.ReactNode;
}> = ({ design, children }) => {
  const { width, height } = useVideoConfig();
  if (sameShape({ width, height }, design)) return <>{children}</>;
  const scale = Math.min(width / design.width, height / design.height);
  return (
    <AbsoluteFill style={{ alignItems: "center", justifyContent: "center", overflow: "hidden" }}>
      <div
        style={{
          position: "relative",
          flexShrink: 0,
          width: design.width,
          height: design.height,
          transform: `scale(${scale})`,
        }}
      >
        <Sequence layout="none" width={design.width} height={design.height}>
          {children}
        </Sequence>
      </div>
    </AbsoluteFill>
  );
};

const TransitionWrap: React.FC<{ item: TimelineItem; children: React.ReactNode }> = ({
  item,
  children,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  useElements([item.enter?.kind, item.exit?.kind]);
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

const noop = () => undefined;

type Direction = "entering" | "exiting";

function transitionAt(
  item: TimelineItem,
  frame: number,
): { ref: TransitionRef; direction: Direction; start: number } | null {
  const { enter, exit, durationInFrames } = item;
  if (enter && frame < enter.durationInFrames)
    return { ref: enter, direction: "entering", start: 0 };
  const exitStart = exit ? durationInFrames - exit.durationInFrames : 0;
  if (exit && frame >= exitStart) return { ref: exit, direction: "exiting", start: exitStart };
  return null;
}

function activeTransition(
  item: TimelineItem,
  frame: number,
): { ref: TransitionRef; direction: Direction; progress: number } | null {
  const at = transitionAt(item, frame);
  if (!at) return null;
  const { ref, direction, start } = at;
  const progress = interpolate(frame, [start, start + ref.durationInFrames], [0, 1], CLAMP_RIGHT);
  return { ref, direction, progress };
}

const NO_SPANS: Span[] = [];

const duckKey = (keys: string[]) => keys.join("|");

const AudioLayers: React.FC<{ spec: TimelineSpec }> = ({ spec }) => {
  const layers = useMemo(() => {
    const spansByKey = new Map<string, Span[]>();
    const spansFor = (keys: string[]) => {
      const key = duckKey(keys);
      const known = spansByKey.get(key);
      if (known) return known;
      const spans = duckingSpans(spec, new Set(keys));
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
  const baseVolume = (typeof item.volume === "number" ? item.volume : 1) * (item.gain ?? 1);
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
      color: "#f87171",
      fontFamily: MAIN_FONT,
      fontSize: 40,
    }}
  >
    Unknown element: {name}
  </AbsoluteFill>
);
