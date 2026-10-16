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
import { remapFrame, type GroupProps } from "../shared/compile/clip";
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
import { clamp } from "../shared/math";

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

export const Tracks: React.FC<{ tracks: TimelineTrack[] }> = ({ tracks }) => (
  <>
    {tracks.map((track) => (
      <AbsoluteFill key={track.id}>
        {track.items.map((item) => (
          <Sequence
            key={item.id}
            from={item.from}
            durationInFrames={item.durationInFrames}
            name={item.component}
            premountFor={
              item.component === "core/group" || item.component === "core/recording"
                ? 30
                : undefined
            }
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
        opacity: clamp(opacity, 0, 1),
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
  if (item.component === "core/group") return <Group {...(item.props as GroupProps)} />;
  return <ElementItem item={item} />;
};

const ElementItem: React.FC<{ item: TimelineItem }> = ({ item }) => {
  const base = useAssetBase();
  const ready = useElements([item.component]);
  if (!isKnownElement(item.component)) return <MissingElement name={item.component} />;
  const component = ready ? getComponent(item.component) : undefined;
  return component ? createElement(component, resolveProps(item.props, base)) : null;
};

export const Group: React.FC<GroupProps> = ({
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
  let inner: React.ReactNode = <Tracks tracks={tracks ?? []} />;
  if (startFrom) {
    inner = (
      <Sequence from={-startFrom} layout="none">
        {inner}
      </Sequence>
    );
  }
  if (loopEvery && loopEvery > 0) inner = <Loop durationInFrames={loopEvery}>{inner}</Loop>;
  if (remap?.length)
    inner = <Freeze frame={Math.max(0, Math.round(remapFrame(remap, frame)))}>{inner}</Freeze>;
  if (crop) {
    const s = 1 / Math.max(0.05, Math.max(crop.w, crop.h));
    inner = (
      <AbsoluteFill style={{ overflow: "hidden" }}>
        <AbsoluteFill
          style={{
            transformOrigin: "0 0",
            transform: `scale(${s}) translate(${-crop.x * 100}%, ${-crop.y * 100}%)`,
          }}
        >
          {inner}
        </AbsoluteFill>
      </AbsoluteFill>
    );
  }
  if (wrapper) {
    const component = getComponent(wrapper.component);
    if (component)
      inner = createElement(
        component,
        resolveProps(wrapper.props, base),
        <AbsoluteFill>{inner}</AbsoluteFill>,
      );
  }
  if (design) inner = <DesignBox design={design}>{inner}</DesignBox>;
  return <AbsoluteFill style={background ? { background } : undefined}>{inner}</AbsoluteFill>;
};

const DesignBox: React.FC<{
  design: { width: number; height: number };
  children: React.ReactNode;
}> = ({ design, children }) => {
  const { width, height } = useVideoConfig();
  if (Math.abs(width / height - design.width / design.height) < 0.01) return <>{children}</>;
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
  return createElement(
    presentation.component as React.ComponentType<Record<string, unknown>>,
    {
      presentationProgress: active.progress,
      presentationDirection: active.direction,
      passedProps: presentation.props,
      presentationDurationInFrames: active.ref.durationInFrames,
      onElementImage: noop,
      onUnmount: noop,
      bothEnteringAndExiting: false,
    },
    children,
  );
};

const noop = () => undefined;

function activeTransition(
  item: TimelineItem,
  frame: number,
): { ref: TransitionRef; direction: "entering" | "exiting"; progress: number } | null {
  const { enter, exit, durationInFrames } = item;
  if (enter && frame < enter.durationInFrames) {
    return {
      ref: enter,
      direction: "entering",
      progress: interpolate(frame, [0, enter.durationInFrames], [0, 1], {
        extrapolateRight: "clamp",
      }),
    };
  }
  if (exit && frame >= durationInFrames - exit.durationInFrames) {
    const start = durationInFrames - exit.durationInFrames;
    return {
      ref: exit,
      direction: "exiting",
      progress: interpolate(frame, [start, durationInFrames], [0, 1], {
        extrapolateRight: "clamp",
      }),
    };
  }
  return null;
}

const NO_SPANS: Span[] = [];

const AudioLayers: React.FC<{ spec: TimelineSpec }> = ({ spec }) => {
  const spansByKeys = useMemo(() => {
    const out = new Map<string, Span[]>();
    for (const a of spec.audio ?? []) {
      if (!a.duck) continue;
      const id = a.duck.keys.join("|");
      if (!out.has(id)) out.set(id, duckingSpans(spec, new Set(a.duck.keys)));
    }
    return out;
  }, [spec]);
  return (
    <>
      {(spec.audio ?? [])
        .filter((a) => a.gain !== 0)
        .map((a) => (
          <AudioLayer
            key={a.id}
            item={a}
            spans={(a.duck && spansByKeys.get(a.duck.keys.join("|"))) || NO_SPANS}
          />
        ))}
    </>
  );
};

const AudioLayer: React.FC<{ item: AudioItem; spans: Span[] }> = ({ item, spans }) => {
  const base = useAssetBase();
  const { durationInFrames: total } = useVideoConfig();
  const duration = item.durationInFrames ?? Math.max(1, total - item.from);
  const baseVolume = (typeof item.volume === "number" ? item.volume : 1) * (item.gain ?? 1);
  const fadeIn = item.fadeInFrames ?? 0;
  const fadeOut = item.fadeOutFrames ?? 0;
  return (
    <Sequence from={item.from} durationInFrames={duration} name={`audio ${item.id}`} layout="none">
      <Audio
        src={resolveAssetRef(item.src, base)}
        loop={item.loop}
        trimBefore={item.trimBefore}
        volume={(f) => {
          const inV = fadeIn
            ? interpolate(f, [0, fadeIn], [0, 1], { extrapolateRight: "clamp" })
            : 1;
          const outV = fadeOut
            ? interpolate(f, [duration - fadeOut, duration], [1, 0], { extrapolateLeft: "clamp" })
            : 1;
          const duck = item.duck
            ? duckGain(item.from + f, spans, item.duck.level, item.duck.fadeFrames)
            : 1;
          const env = envelopeAt(item.envelope?.volume, f, 1);
          return Math.max(0, baseVolume * Math.min(inV, outV) * duck * env);
        }}
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
      fontFamily: "system-ui",
      fontSize: 40,
    }}
  >
    Unknown element: {name}
  </AbsoluteFill>
);
