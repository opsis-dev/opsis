import { Audio } from "@remotion/media";
import React, { createElement } from "react";
import {
  AbsoluteFill,
  Freeze,
  interpolate,
  Loop,
  Sequence,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";
import {
  AssetProvider,
  DEFAULT_QUALITY,
  DEFAULT_REFRAME,
  QualityProvider,
  ReframeProvider,
  resolveAssetRef,
  resolveProps,
  useAssetBase,
} from "#video/assets.tsx";
import { groupOf } from "#video/common/clip.ts";
import { ERROR_COLOR, STAGE_COLOR } from "#video/common/element-colors.ts";
import { envelopeAt } from "#video/common/envelope.ts";
import { CODE_FONT, MAIN_FONT } from "#video/common/fonts.ts";
import { percent } from "#video/common/format.ts";
import { MIN_CROP } from "#video/common/limits.ts";
import { clamp01, EPSILON } from "#video/common/math.ts";
import { remapFrame } from "#video/common/remap.ts";
import type { ElementItem } from "#video/common/spec.ts";
import { CLAMP_RIGHT } from "#video/core/anim.ts";
import { RecordingClip } from "#video/core/recording-clip.tsx";
import type { Size } from "#video/core/still-layout.ts";
import { CutBoardProvider, useCutBoard } from "#video/core/transition-cuts.tsx";
import { FontProvider } from "#video/fonts/font-provider.tsx";
import type {
  Rect,
  GroupProps,
  SequencerProps,
  TimelineItem,
  TimelineSpec,
  TimelineTrack,
  TransitionRef,
  VisualEnvelope,
} from "#video/generated/spec.ts";
import { PaintRule } from "#video/registry/framing.tsx";
import {
  getComponent,
  getPresentation,
  getSequencer,
  useElement,
  useResolvedElement,
} from "#video/registry/index.ts";

const ELEMENT_FONTS: React.CSSProperties &
  Record<"--font-geist-sans" | "--font-geist-mono", string> = {
  "--font-geist-sans": MAIN_FONT,
  "--font-geist-mono": CODE_FONT,
};

const PREMOUNT_FRAMES = 30;

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
          {spec.mix ? <MasterStem src={spec.mix.src} /> : null}
        </AbsoluteFill>
      </QualityProvider>
    </ReframeProvider>
  </AssetProvider>
);

const Tracks: React.FC<{ tracks: TimelineTrack[] }> = ({ tracks }) => (
  <CutBoardProvider>
    {tracks.map((track) => (
      <AbsoluteFill key={track.id}>
        {track.items.map((item) => (
          <Sequence
            key={item.id}
            from={item.from}
            durationInFrames={item.durationInFrames}
            name={item.kind === "element" ? item.component : item.kind}
            premountFor={item.kind === "element" ? undefined : PREMOUNT_FRAMES}
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
  </CutBoardProvider>
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
  if (item.kind === "group") return <Group {...item.props} />;
  if (item.kind === "recording") return <RecordingClip {...item.props} />;
  return <Element item={item} />;
};

const Element: React.FC<{ item: ElementItem }> = ({ item }) => {
  const base = useAssetBase();
  const element = useResolvedElement(item.component, getComponent);
  if (element?.status === "unknown") return <MissingElement name={item.component} />;
  return element?.status === "ready" ? (
    <DesignBox design={item.design} fill={!item.design?.exact}>
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

const TransitionWrap: React.FC<{ item: TimelineItem; children: React.ReactNode }> = ({
  item,
  children,
}) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const board = useCutBoard();
  useElement(item.enter?.kind);
  useElement(item.exit?.kind);
  const active = activeTransition(item, frame);
  if (!active) return <>{children}</>;
  const presentation = getPresentation(active.ref.kind, active.ref.params ?? {}, { width, height });
  if (!presentation) return <>{children}</>;
  const Presentation = presentation.component;
  const { direction, progress } = active;
  const cut =
    direction === "entering"
      ? item.from
      : item.from + item.durationInFrames - active.ref.durationInFrames;
  return (
    <Presentation
      presentationProgress={progress}
      presentationDirection={direction}
      passedProps={presentation.props}
      presentationDurationInFrames={active.ref.durationInFrames}
      onElementImage={(image, draw) => board?.report(cut, direction, { image, progress, draw })}
      onUnmount={() => board?.leave(cut, direction)}
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
