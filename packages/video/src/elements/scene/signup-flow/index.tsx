import type { CSSProperties } from "react";
import { fontOr } from "#video/common/fonts.ts";
import { themeColors } from "#video/common/theme.ts";
import { BlurIn } from "#video/elements/scene/signup-flow/blur-in.tsx";
import { Cursor } from "#video/elements/scene/signup-flow/cursor.tsx";
import {
  Field,
  FieldControl,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "#video/elements/scene/signup-flow/field.tsx";
import { useBlurInTransition } from "#video/elements/scene/signup-flow/use-blur-in-transition.ts";
import { useCursorPath } from "#video/elements/scene/signup-flow/use-cursor-path.ts";
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

type SignupFlowViewProps = Omit<ElementSettings["signup-flow"], "theme"> & {
  theme?: Partial<NeutralTheme>;
};

const NOTICE_PLACE: Record<Corner, CSSProperties> = {
  "bottom-left": { left: 32, bottom: 32 },
  "bottom-right": { right: 32, bottom: 32 },
  "top-left": { left: 32, top: 32 },
  "top-right": { right: 32, top: 32 },
};

const STAGE_W = 1280;
const CARD_W = 376;
const CARD_TOP = 48;
const CARD_LEFT = (STAGE_W - CARD_W) / 2;
const CENTER_X = STAGE_W / 2;

const NAME_Y = 216;
const EMAIL_Y = 296;
const PASS_Y = 398;
const CONFIRM_Y = 500;
const CREATE_Y = 564;

function SignupFlowView({
  title,
  description,
  fullName,
  email,
  password,
  createLabel,
  toastTitle,
  theme,
  nameLabel,
  emailLabel,
  emailHelp,
  passwordLabel,
  passwordHelp,
  confirmLabel,
  speed,
  loadingSeconds: loading,
  noticeSeconds: notice,
  cursorShape,
  cursorSize,
  noticeCorner,
  radius,
  fontFamily,
  backgroundColor,
}: SignupFlowViewProps) {
  const resolved = useNeutralTheme(theme);
  const opts = { theme, speed };
  const blur = { speed };

  const cardEnter = useBlurInTransition([{ at: 0, state: "revealed", duration: 18 }], {
    distance: 0,
    speed,
  });
  const enterHeader = useBlurInTransition([{ at: 18, state: "revealed", duration: 16 }], blur);
  const enterName = useBlurInTransition([{ at: 24, state: "revealed", duration: 16 }], blur);
  const enterEmail = useBlurInTransition([{ at: 30, state: "revealed", duration: 16 }], blur);
  const enterPass = useBlurInTransition([{ at: 36, state: "revealed", duration: 16 }], blur);
  const enterConfirm = useBlurInTransition([{ at: 42, state: "revealed", duration: 16 }], blur);
  const enterButton = useBlurInTransition([{ at: 48, state: "revealed", duration: 16 }], blur);

  const DEMO = 48;
  const successAt = 192 + DEMO + loading;

  const cursorStyle = useCursorPath(
    [
      { at: 0, x: 160, y: 120 },
      { at: 18 + DEMO, x: CENTER_X, y: NAME_Y, duration: 18, click: true },
      { at: 52 + DEMO, x: CENTER_X, y: EMAIL_Y, duration: 30, click: true },
      { at: 96 + DEMO, x: CENTER_X, y: PASS_Y, duration: 40, click: true },
      { at: 134 + DEMO, x: CENTER_X, y: CONFIRM_Y, duration: 32, click: true },
      { at: 176 + DEMO, x: CENTER_X, y: CREATE_Y, duration: 38, click: true },
    ],
    { speed },
  );

  const nameStyle = useInputTransition(
    [
      { at: 18 + DEMO, state: "active", duration: 6 },
      { at: 20 + DEMO, state: "typing", duration: 20 },
      { at: 52 + DEMO, state: "blur", duration: 8 },
    ],
    opts,
  );
  const emailStyle = useInputTransition(
    [
      { at: 52 + DEMO, state: "active", duration: 6 },
      { at: 54 + DEMO, state: "typing", duration: 28 },
      { at: 96 + DEMO, state: "blur", duration: 8 },
    ],
    opts,
  );
  const passStyle = useInputTransition(
    [
      { at: 96 + DEMO, state: "active", duration: 6 },
      { at: 98 + DEMO, state: "typing", duration: 22 },
      { at: 134 + DEMO, state: "blur", duration: 8 },
    ],
    opts,
  );
  const confirmStyle = useInputTransition(
    [
      { at: 134 + DEMO, state: "active", duration: 6 },
      { at: 136 + DEMO, state: "typing", duration: 22 },
      { at: 176 + DEMO, state: "blur", duration: 8 },
    ],
    opts,
  );

  const buttonStyle = useButtonTransition(
    [
      { at: 176 + DEMO, state: "hover", duration: 8 },
      { at: 186 + DEMO, state: "press", duration: 6 },
      { at: 192 + DEMO, state: "loading", duration: 6 },
      { at: successAt, state: "success", duration: 16 },
    ],
    opts,
  );

  const toastStyle = useToastTransition(
    [
      { at: successAt, state: "visible", duration: 14 },
      { at: successAt + notice, state: "hidden", duration: 14 },
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
        fontFamily: fontOr(fontFamily, "main"),
      }}
    >
      <div
        style={{
          position: "absolute",
          left: CARD_LEFT,
          top: CARD_TOP,
          width: CARD_W,
          height: 580,
          boxSizing: "border-box",
          padding: 28,
          display: "flex",
          flexDirection: "column",
          gap: 24,
          background: resolved.background,
          border: `1px solid ${resolved.border}`,
          borderRadius: radius + 4,
          boxShadow: "0 10px 30px -12px rgba(0,0,0,0.22), 0 2px 8px -3px rgba(0,0,0,0.10)",
          opacity: cardEnter.opacity,
          filter: cardEnter.blur > 0 ? `blur(${cardEnter.blur}px)` : "none",
        }}
      >
        <BlurIn display="block" style={enterHeader}>
          <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
            <div
              style={{
                fontSize: 22,
                lineHeight: "28px",
                fontWeight: 600,
                letterSpacing: "-0.02em",
                color: resolved.cardForeground,
              }}
            >
              {title}
            </div>
            <div
              style={{
                fontSize: 14,
                lineHeight: "20px",
                color: resolved.mutedForeground,
              }}
            >
              {description}
            </div>
          </div>
        </BlurIn>

        <FieldGroup gap={16}>
          <BlurIn display="block" style={enterName}>
            <Field>
              <FieldLabel theme={theme}>{nameLabel}</FieldLabel>
              <FieldControl>
                <Input placeholder={fullName} value={fullName} style={nameStyle} theme={theme} />
              </FieldControl>
            </Field>
          </BlurIn>

          <BlurIn display="block" style={enterEmail}>
            <Field>
              <FieldLabel theme={theme}>{emailLabel}</FieldLabel>
              <FieldControl>
                <Input placeholder={email} value={email} style={emailStyle} theme={theme} />
              </FieldControl>
              <FieldDescription theme={theme}>{emailHelp}</FieldDescription>
            </Field>
          </BlurIn>

          <BlurIn display="block" style={enterPass}>
            <Field>
              <FieldLabel theme={theme}>{passwordLabel}</FieldLabel>
              <FieldControl>
                <Input placeholder={password} value={password} style={passStyle} theme={theme} />
              </FieldControl>
              <FieldDescription theme={theme}>{passwordHelp}</FieldDescription>
            </Field>
          </BlurIn>

          <BlurIn display="block" style={enterConfirm}>
            <Field>
              <FieldLabel theme={theme}>{confirmLabel}</FieldLabel>
              <FieldControl>
                <Input placeholder={password} value={password} style={confirmStyle} theme={theme} />
              </FieldControl>
            </Field>
          </BlurIn>
        </FieldGroup>

        <BlurIn display="block" style={enterButton}>
          <Field gap={10}>
            <FieldControl>
              <Button label={createLabel} style={buttonStyle} theme={theme} />
            </FieldControl>
          </Field>
        </BlurIn>
      </div>

      <div style={{ position: "absolute", ...NOTICE_PLACE[noticeCorner] }}>
        <Toast title={toastTitle} variant="success" style={toastStyle} theme={theme} />
      </div>

      <Cursor style={cursorStyle} variant={cursorShape} size={cursorSize} theme={theme} />
    </div>
  );
}

export function SignupFlow({ theme, radius, ...settings }: ElementSettings["signup-flow"]) {
  const tokens = {
    ...(theme.mode === "dark" ? defaultDarkTheme : defaultLightTheme),
    ...themeColors(theme),
    radius,
  };
  return (
    <NeutralUIProvider mode={theme.mode} theme={tokens}>
      <SignupFlowView {...settings} radius={radius} theme={tokens} />
    </NeutralUIProvider>
  );
}
