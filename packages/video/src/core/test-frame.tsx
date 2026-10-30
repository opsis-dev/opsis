import { type ComponentType, useLayoutEffect, useMemo } from "react";
import { Internals } from "remotion";

const COMPOSITION_ID = "test-frame";

const ENVIRONMENT = {
  isPlayer: true,
  isRendering: false,
  isStudio: false,
  isClientSideRendering: false,
  isReadOnlyStudio: false,
};

const SCALE = { type: "scale", scale: 1 } as const;

const DRAWN_ELSEWHERE = () => null;

type TestFrameProps<P extends object> = {
  component: ComponentType<P>;
  inputProps?: P;
  durationInFrames: number;
  fps: number;
  compositionWidth: number;
  compositionHeight: number;
  frameToDisplay: number;
};

export function TestFrame<P extends object>({
  component: Component,
  inputProps,
  durationInFrames,
  fps,
  compositionWidth: width,
  compositionHeight: height,
  frameToDisplay,
}: TestFrameProps<P>) {
  useLayoutEffect(() => {
    window.remotion_isPlayer = true;
  }, []);
  const manager = useMemo(
    () => ({
      compositions: [
        {
          component: DRAWN_ELSEWHERE,
          durationInFrames,
          width,
          height,
          fps,
          id: COMPOSITION_ID,
          order: null,
          folderName: null,
          parentFolderName: null,
          schema: null,
          calculateMetadata: null,
          stack: null,
        },
      ],
      folders: [],
      currentAssetMetadata: null,
      currentCompositionMetadata: {
        defaultCodec: null,
        defaultOutName: null,
        defaultPixelFormat: null,
        defaultProResProfile: null,
        defaultSampleRate: null,
        defaultVideoImageFormat: null,
        durationInFrames,
        fps,
        height,
        width,
        props: {},
      },
      canvasContent: { type: "composition", compositionId: COMPOSITION_ID } as const,
    }),
    [durationInFrames, fps, height, width],
  );
  const frameState = useMemo(() => ({ [COMPOSITION_ID]: frameToDisplay }), [frameToDisplay]);
  return (
    <Internals.RemotionEnvironmentContext.Provider value={ENVIRONMENT}>
      <Internals.CanUseRemotionHooksProvider>
        <Internals.CompositionManager.Provider value={manager}>
          <Internals.RemotionRootContexts
            numberOfAudioTags={0}
            logLevel="error"
            audioLatencyHint="playback"
            previewSampleRate={48000}
            videoEnabled
            audioEnabled={false}
            frameState={frameState}
            _experimentalKeepAudioContextAlive={false}
          >
            <Internals.CurrentScaleContext.Provider value={SCALE}>
              <Component {...(inputProps ?? ({} as P))} />
            </Internals.CurrentScaleContext.Provider>
          </Internals.RemotionRootContexts>
        </Internals.CompositionManager.Provider>
      </Internals.CanUseRemotionHooksProvider>
    </Internals.RemotionEnvironmentContext.Provider>
  );
}
