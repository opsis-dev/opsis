import type { CSSProperties } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { themeColors } from "#video/common/theme.ts";
import { SkeletonBlock } from "#video/elements/scene/ai-prompt-flow/skeleton-block.tsx";
import { Skeleton } from "#video/elements/scene/ai-prompt-flow/skeleton.tsx";
import { useSkeletonTransition } from "#video/elements/scene/ai-prompt-flow/use-skeleton-transition.ts";
import type { Corner, ElementSettings } from "#video/generated/spec.ts";
import { type NeutralTheme, useNeutralTheme } from "#video/lib/neutral-ui/index.ts";
import {
  NeutralUIProvider,
  defaultDarkTheme,
  defaultLightTheme,
} from "#video/lib/neutral-ui/theme.ts";
import { Button } from "#video/parts/button.tsx";
import { Input } from "#video/parts/input.tsx";
import { Toast } from "#video/parts/toast.tsx";
import { useButtonTransition } from "#video/parts/use-button-transition.ts";
import { useInputTransition } from "#video/parts/use-input-transition.ts";
import { useToastTransition } from "#video/parts/use-toast-transition.ts";

type AiPromptFlowViewProps = Omit<ElementSettings["ai-prompt-flow"], "theme" | "radius"> & {
  theme?: Partial<NeutralTheme>;
};

const NOTICE_PLACE: Record<Corner, CSSProperties> = {
  "bottom-left": { left: 32, bottom: 32 },
  "bottom-right": { right: 32, bottom: 32 },
  "top-left": { left: 32, top: 32 },
  "top-right": { right: 32, top: 32 },
};

const STAGE_W = 1280;
const PROMPT_W = 520;
const PROMPT_LEFT = (STAGE_W - PROMPT_W) / 2;
const PROMPT_TOP = 168;
const PROMPT_H = 92;

const BTN_W = 200;
const BTN_LEFT = (STAGE_W - BTN_W) / 2;
const BTN_TOP = 278;
const BTN_H = 64;

const ANSWER_TOP = 380;

function AiPromptFlowView({
  prompt,
  buttonLabel,
  answerLines,
  toastTitle,
  theme,
  typingSeconds: typing,
  loadingSeconds: loading,
  noticeSeconds: notice,
  speed,
  fieldWidth,
  answerWidth,
  answerSize,
  fontFamily,
  buttonVariant,
  noticeCorner,
  backgroundColor,
}: AiPromptFlowViewProps) {
  const resolved = useNeutralTheme(theme);
  const opts = { theme, speed };
  const pressAt = typing + 2;
  const loadedAt = pressAt + 12 + loading;
  const noticeAt = loadedAt + 10;

  const inputStyle = useInputTransition(
    [
      { at: 0, state: "active", duration: 1 },
      { at: 0, state: "typing", duration: typing },
    ],
    opts,
  );

  const buttonStyle = useButtonTransition(
    [
      { at: pressAt, state: "hover", duration: 6 },
      { at: pressAt + 6, state: "press", duration: 4 },
      { at: pressAt + 10, state: "loading", duration: 4 },
    ],
    opts,
  );

  const skeletonStyle = useSkeletonTransition(
    [
      { at: pressAt + 12, state: "loading", duration: 1 },
      { at: loadedAt, state: "loaded", duration: 16 },
    ],
    { speed },
  );

  const panelOpacity = buttonStyle.spinnerOpacity;

  const toastStyle = useToastTransition(
    [
      { at: noticeAt, state: "visible", duration: 14 },
      { at: noticeAt + notice, state: "hidden", duration: 14 },
    ],
    { speed },
  );

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        background: backgroundColor || "transparent",
      }}
    >
      <div
        style={{
          position: "absolute",
          left: PROMPT_LEFT,
          top: PROMPT_TOP,
          width: PROMPT_W,
          height: PROMPT_H,
        }}
      >
        <Input
          placeholder="Ask anything…"
          value={prompt}
          style={inputStyle}
          theme={theme}
          fieldWidth={fieldWidth}
        />
      </div>

      <div
        style={{
          position: "absolute",
          left: BTN_LEFT,
          top: BTN_TOP,
          width: BTN_W,
          height: BTN_H,
        }}
      >
        <Button label={buttonLabel} variant={buttonVariant} style={buttonStyle} theme={theme} />
      </div>

      <div
        style={{
          position: "absolute",
          left: (STAGE_W - answerWidth) / 2,
          top: ANSWER_TOP,
          width: answerWidth,
          display: "flex",
          justifyContent: "center",
          opacity: panelOpacity,
        }}
      >
        <Skeleton
          style={skeletonStyle}
          theme={theme}
          placeholder={
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 14,
                width: answerWidth,
              }}
            >
              {answerLines.map((_, i) => (
                <SkeletonBlock
                  key={i}
                  width={i === answerLines.length - 1 ? "70%" : "100%"}
                  height={answerSize}
                  baseColor={resolved.muted}
                />
              ))}
            </div>
          }
        >
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 8,
              width: answerWidth,
              fontFamily: fontOr(fontFamily, "main"),
              fontSize: answerSize,
              lineHeight: 1.45,
              letterSpacing: "-0.01em",
              color: resolved.foreground,
            }}
          >
            {answerLines.map((line, i) => (
              <span key={i}>{line}</span>
            ))}
          </div>
        </Skeleton>
      </div>

      <div style={{ position: "absolute", ...NOTICE_PLACE[noticeCorner] }}>
        <Toast title={toastTitle} variant="success" style={toastStyle} theme={theme} />
      </div>
    </div>
  );
}

export function AiPromptFlow({ theme, radius, ...settings }: ElementSettings["ai-prompt-flow"]) {
  const tokens = {
    ...(theme.mode === "dark" ? defaultDarkTheme : defaultLightTheme),
    ...themeColors(theme),
    radius,
  };
  return (
    <NeutralUIProvider mode={theme.mode} theme={tokens}>
      <AiPromptFlowView {...settings} theme={tokens} />
    </NeutralUIProvider>
  );
}
