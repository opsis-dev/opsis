import React from "react";
import { AbsoluteFill } from "remotion";
import { useDesignScale } from "./lib/design-scale";

export const CHROME_H = 38;
const WINDOW_RADIUS = 14;
const PILL_RADIUS = 9999;
const WINDOW_FILTER = "saturate(1.05) contrast(1.025) brightness(1.01)";
const SYSTEM_FONT = 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif';

export const Stage: React.FC<{ children: React.ReactNode }> = ({ children }) => (
  <AbsoluteFill
    style={{
      justifyContent: "center",
      alignItems: "center",
      overflow: "hidden",
    }}
  >
    {children}
  </AbsoluteFill>
);

const GLOW_RADIUS = 60;
const GLOW_BLUR = 90;
const GLOW_SCALE = "0.97, 0.96";
const RIM_RADIUS = WINDOW_RADIUS + 1;

const RIM_LOOKS = {
  light: { glow: "rgba(30,20,10,0.30)", lift: "2.5%", rim: "rgba(15,23,42,0.16)" },
  dark: { glow: "rgba(255,255,255,0.10)", lift: "2%", rim: "rgba(255,255,255,0.14)" },
} as const;

export const RimLight: React.FC<{ light?: boolean }> = ({ light = false }) => {
  const k = useDesignScale();
  const look = RIM_LOOKS[light ? "light" : "dark"];
  return (
    <>
      <div
        style={{
          position: "absolute",
          inset: 0,
          borderRadius: GLOW_RADIUS * k,
          background: look.glow,
          transform: `translateY(${look.lift}) scale(${GLOW_SCALE})`,
          filter: `blur(${GLOW_BLUR * k}px)`,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: -k,
          borderRadius: RIM_RADIUS * k,
          boxShadow: `0 0 0 ${k}px ${look.rim}`,
          pointerEvents: "none",
        }}
      />
    </>
  );
};

export type WindowStyle = "macos" | "windows" | "linux";
export type FrameTheme = "light" | "dark";

const FRAME_COLORS = {
  light: {
    bar: "linear-gradient(180deg, #fafbfc 0%, #f0f2f5 100%)",
    edge: "rgba(15, 23, 42, 0.07)",
    field: "rgba(15, 23, 42, 0.045)",
    fieldEdge: "rgba(15, 23, 42, 0.05)",
    text: "rgba(15, 23, 42, 0.5)",
    icon: "rgba(15, 23, 42, 0.62)",
    button: "rgba(15, 23, 42, 0.07)",
  },
  dark: {
    bar: "linear-gradient(180deg, #2c2c31 0%, #232328 100%)",
    edge: "rgba(0, 0, 0, 0.45)",
    field: "rgba(255, 255, 255, 0.07)",
    fieldEdge: "rgba(255, 255, 255, 0.06)",
    text: "rgba(255, 255, 255, 0.62)",
    icon: "rgba(255, 255, 255, 0.78)",
    button: "rgba(255, 255, 255, 0.1)",
  },
} as const;

const TRAFFIC_LIGHT = { size: 11, gap: 7 } as const;

const TRAFFIC_LIGHTS = [
  ["#ff5f57", "#e0443e"],
  ["#febc2e", "#dea123"],
  ["#28c840", "#1aab29"],
] as const;

const TrafficLights: React.FC = () => {
  const k = useDesignScale();
  return (
    <div style={{ display: "flex", gap: TRAFFIC_LIGHT.gap * k, flexShrink: 0 }}>
      {TRAFFIC_LIGHTS.map(([fill, rim]) => (
        <div
          key={fill}
          style={{
            width: TRAFFIC_LIGHT.size * k,
            height: TRAFFIC_LIGHT.size * k,
            borderRadius: PILL_RADIUS,
            background: `radial-gradient(circle at 35% 30%, ${fill} 0%, ${rim} 100%)`,
            boxShadow: `inset 0 0 0 ${0.5 * k}px rgba(0,0,0,0.12)`,
          }}
        />
      ))}
    </div>
  );
};

const ICONS = {
  minimize: "M3 8h10",
  maximize: "M3.5 3.5h9v9h-9z",
  close: "M4 4l8 8M12 4l-8 8",
} as const;

const BUTTON_NAMES = ["minimize", "maximize", "close"] as const;

const Glyph: React.FC<{ d: string; size: number; color: string }> = ({ d, size, color }) => (
  <svg width={size} height={size} viewBox="0 0 16 16">
    <path d={d} stroke={color} strokeWidth={1.2} fill="none" strokeLinecap="round" />
  </svg>
);

type FrameColors = (typeof FRAME_COLORS)[FrameTheme];

type ButtonLook = (
  k: number,
  c: FrameColors,
) => {
  row: React.CSSProperties;
  button: React.CSSProperties;
  glyph: number;
};

type Slot = "lights" | { spacer: number } | { buttons: ButtonLook } | null;

type StyleLook = {
  windowRadius: number;
  fieldRadius: number;
  fieldAlign: "center" | "flex-start";
  lead: Slot;
  trail: Slot;
};

const TITLE_BAR = { gap: 10, padX: 14 } as const;

const ADDRESS_FIELD = { height: 22, fontSize: 11.5, padX: 12 } as const;

const STYLE_LOOKS: Record<WindowStyle, StyleLook> = {
  macos: {
    windowRadius: WINDOW_RADIUS,
    fieldRadius: 7,
    fieldAlign: "center",
    lead: "lights",
    trail: { spacer: 52 },
  },
  linux: {
    windowRadius: WINDOW_RADIUS,
    fieldRadius: 7,
    fieldAlign: "center",
    lead: { spacer: 88 },
    trail: {
      buttons: (k, c) => ({
        row: { gap: 9 * k },
        button: { width: 20 * k, height: 20 * k, borderRadius: PILL_RADIUS, background: c.button },
        glyph: 11 * k,
      }),
    },
  },
  windows: {
    windowRadius: 8,
    fieldRadius: PILL_RADIUS,
    fieldAlign: "flex-start",
    lead: null,
    trail: {
      buttons: (k) => ({
        row: { alignSelf: "stretch", marginRight: -TITLE_BAR.padX * k },
        button: { width: 44 * k, alignSelf: "stretch" },
        glyph: 12 * k,
      }),
    },
  },
};

const WindowButtons: React.FC<{ buttonLook: ButtonLook; theme: FrameTheme }> = ({
  buttonLook,
  theme,
}) => {
  const k = useDesignScale();
  const c = FRAME_COLORS[theme];
  const look = buttonLook(k, c);
  return (
    <div style={{ display: "flex", flexShrink: 0, ...look.row }}>
      {BUTTON_NAMES.map((n) => (
        <div key={n} style={{ display: "grid", placeItems: "center", ...look.button }}>
          <Glyph d={ICONS[n]} size={look.glyph} color={c.icon} />
        </div>
      ))}
    </div>
  );
};

const SlotView: React.FC<{ slot: Slot; theme: FrameTheme }> = ({ slot, theme }) => {
  const k = useDesignScale();
  if (slot === null) return null;
  if (slot === "lights") return <TrafficLights />;
  if ("buttons" in slot) return <WindowButtons buttonLook={slot.buttons} theme={theme} />;
  return <div style={{ width: slot.spacer * k, flexShrink: 0 }} />;
};

const AddressField: React.FC<{ look: StyleLook; theme: FrameTheme; hostLabel: string }> = ({
  look,
  theme,
  hostLabel,
}) => {
  const k = useDesignScale();
  const c = FRAME_COLORS[theme];
  return (
    <div
      style={{
        flex: 1,
        height: ADDRESS_FIELD.height * k,
        borderRadius: look.fieldRadius * k,
        background: c.field,
        border: `${k}px solid ${c.fieldEdge}`,
        color: c.text,
        fontSize: ADDRESS_FIELD.fontSize * k,
        fontFamily: SYSTEM_FONT,
        fontWeight: 450,
        letterSpacing: "0.01em",
        display: "flex",
        alignItems: "center",
        justifyContent: look.fieldAlign,
        padding: `0 ${ADDRESS_FIELD.padX * k}px`,
        overflow: "hidden",
        whiteSpace: "nowrap",
        textOverflow: "ellipsis",
      }}
    >
      {hostLabel}
    </div>
  );
};

const TitleBar: React.FC<{
  windowStyle: WindowStyle;
  theme: FrameTheme;
  addressBar: boolean;
  hostLabel: string;
}> = ({ windowStyle, theme, addressBar, hostLabel }) => {
  const k = useDesignScale();
  const c = FRAME_COLORS[theme];
  const look = STYLE_LOOKS[windowStyle];
  return (
    <div
      style={{
        flex: `0 0 ${CHROME_H * k}px`,
        display: "flex",
        alignItems: "center",
        gap: TITLE_BAR.gap * k,
        padding: `0 ${TITLE_BAR.padX * k}px`,
        background: c.bar,
        borderBottom: `1px solid ${c.edge}`,
      }}
    >
      <SlotView slot={look.lead} theme={theme} />
      {addressBar ? (
        <AddressField look={look} theme={theme} hostLabel={hostLabel} />
      ) : (
        <div style={{ flex: 1 }} />
      )}
      <SlotView slot={look.trail} theme={theme} />
    </div>
  );
};

export const WindowFrame: React.FC<{
  chrome?: boolean;
  hostLabel?: string;
  windowStyle?: WindowStyle;
  theme?: FrameTheme;
  addressBar?: boolean;
  children: React.ReactNode;
}> = ({
  chrome = false,
  hostLabel = "",
  windowStyle = "macos",
  theme = "light",
  addressBar = true,
  children,
}) => {
  const k = useDesignScale();
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        borderRadius: STYLE_LOOKS[windowStyle].windowRadius * k,
        overflow: "hidden",
        background: "#fff",
        display: "flex",
        flexDirection: "column",
        filter: WINDOW_FILTER,
        boxShadow: `inset 0 ${k}px 0 rgba(255,255,255,0.9)`,
      }}
    >
      {chrome ? (
        <TitleBar
          windowStyle={windowStyle}
          theme={theme}
          addressBar={addressBar}
          hostLabel={hostLabel}
        />
      ) : null}

      <div
        style={{
          flex: 1,
          position: "relative",
          overflow: "hidden",
          minHeight: 0,
        }}
      >
        {children}
      </div>
    </div>
  );
};
