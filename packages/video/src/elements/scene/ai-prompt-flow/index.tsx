import { Button } from "#video/parts/button.tsx";
import { Input } from "#video/parts/input.tsx";
import { Skeleton } from "#video/elements/scene/ai-prompt-flow/skeleton.tsx";
import { SkeletonBlock } from "#video/elements/scene/ai-prompt-flow/skeleton-block.tsx";
import { Toast } from "#video/parts/toast.tsx";
import { useButtonTransition } from "#video/parts/use-button-transition.ts";
import { useInputTransition } from "#video/parts/use-input-transition.ts";
import { useSkeletonTransition } from "#video/elements/scene/ai-prompt-flow/use-skeleton-transition.ts";
import { useToastTransition } from "#video/parts/use-toast-transition.ts";
import { type NeutralTheme, useNeutralTheme } from "#video/lib/neutral-ui/index.ts";
import { type ElementProps } from "#video/common/spec.ts";
import { isThemeValue, themeColors } from "#video/common/theme.ts";
import { NeutralUIProvider, defaultDarkTheme, defaultLightTheme } from "#video/lib/neutral-ui/theme.ts";
import type React from "react";

export interface AiPromptFlowProps {
  prompt?: string;
  buttonLabel?: string;
  answerLines?: string[];
  toastTitle?: string;
  theme?: Partial<NeutralTheme>;
}

const STAGE_W = 1280;
const PROMPT_W = 520;
const PROMPT_LEFT = (STAGE_W - PROMPT_W) / 2;
const PROMPT_TOP = 168;
const PROMPT_H = 92;

const BTN_W = 200;
const BTN_LEFT = (STAGE_W - BTN_W) / 2;
const BTN_TOP = 278;
const BTN_H = 64;

const ANSWER_W = 560;
const ANSWER_LEFT = (STAGE_W - ANSWER_W) / 2;
const ANSWER_TOP = 380;

const DEFAULT_ANSWER = [
  "The thread debates the Q3 roadmap: ship the editor first,",
  "defer billing to Q4, and pull the migration forward so",
  "infra is unblocked before the team scales next quarter.",
];

function AiPromptFlowView({
  prompt = "Summarize this thread",
  buttonLabel = "Generate",
  answerLines = DEFAULT_ANSWER,
  toastTitle = "Response ready",
  theme,
}: AiPromptFlowProps) {
  const resolved = useNeutralTheme(theme);
  const opts = { theme };

  const inputStyle = useInputTransition(
    [
      { at: 0, state: "active", duration: 1 },
      { at: 0, state: "typing", duration: 50 },
    ],
    opts,
  );

  const buttonStyle = useButtonTransition(
    [
      { at: 52, state: "hover", duration: 6 },
      { at: 58, state: "press", duration: 4 },
      { at: 62, state: "loading", duration: 4 },
    ],
    opts,
  );

  const skeletonStyle = useSkeletonTransition(
    [
      { at: 64, state: "loading", duration: 1 },
      { at: 150, state: "loaded", duration: 16 },
    ],
    {},
  );

  const panelOpacity = buttonStyle.spinnerOpacity;

  const toastStyle = useToastTransition(
    [
      { at: 160, state: "visible", duration: 14 },
      { at: 220, state: "hidden", duration: 14 },
    ],
    {},
  );

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        background: "transparent",
      }}
    >
      {}
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
        />
      </div>

      {}
      <div
        style={{
          position: "absolute",
          left: BTN_LEFT,
          top: BTN_TOP,
          width: BTN_W,
          height: BTN_H,
        }}
      >
        <Button label={buttonLabel} style={buttonStyle} theme={theme} />
      </div>

      {}
      <div
        style={{
          position: "absolute",
          left: ANSWER_LEFT,
          top: ANSWER_TOP,
          width: ANSWER_W,
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
                width: ANSWER_W,
              }}
            >
              {answerLines.map((_, i) => (
                <SkeletonBlock
                  key={i}
                  width={i === answerLines.length - 1 ? "70%" : "100%"}
                  height={18}
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
              width: ANSWER_W,
              fontFamily:
                "var(--font-geist-sans), -apple-system, BlinkMacSystemFont, sans-serif",
              fontSize: 18,
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

      {}
      <div style={{ position: "absolute", right: 32, bottom: 32 }}>
        <Toast
          title={toastTitle}
          variant="success"
          style={toastStyle}
          theme={theme}
        />
      </div>
    </div>
  );
}

export function AiPromptFlow(props: ElementProps & { children?: React.ReactNode }) {
  const { theme } = props;
  if (!isThemeValue(theme)) return <AiPromptFlowView {...(props as unknown as React.ComponentProps<typeof AiPromptFlowView>)} />;
  const tokens = { ...(theme.mode === "dark" ? defaultDarkTheme : defaultLightTheme), ...themeColors(theme) };
  return <NeutralUIProvider mode={theme.mode} theme={tokens}><AiPromptFlowView {...(props as unknown as React.ComponentProps<typeof AiPromptFlowView>)} theme={tokens} /></NeutralUIProvider>;
}
