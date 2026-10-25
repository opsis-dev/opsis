import {
  DEFAULT_ADDRESS_BAR,
  DEFAULT_FRAME_THEME,
  DEFAULT_WINDOW_STYLE,
  type FrameTheme,
  type WindowStyle,
} from "@opsis/core/window-styles";
import React from "react";
import { useDesignScale } from "./lib/design-scale";
import { ink, WINDOW_RADIUS } from "./lib/style";

export const CHROME_H = 38;
const PILL_RADIUS = 9999;
const WINDOW_FILTER = "saturate(1.05) contrast(1.025) brightness(1.01)";
const SYSTEM_FONT = 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif';

const FRAME_COLORS = {
  light: {
    bar: "linear-gradient(180deg, #fafbfc 0%, #f0f2f5 100%)",
    edge: ink(0.07),
    field: ink(0.045),
    fieldEdge: ink(0.05),
    text: ink(0.5),
    icon: ink(0.62),
    button: ink(0.07),
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

const Glyph: React.FC<{ d: string; size: number; color: string }> = ({ d, size, color }) => (
  <svg width={size} height={size} viewBox="0 0 16 16">
    <path d={d} stroke={color} strokeWidth={1.2} fill="none" strokeLinecap="round" />
  </svg>
);

type ButtonsLook = { size: number; gap: number; glyph: number; fullHeight: boolean };

type Slot =
  | { kind: "lights" }
  | { kind: "buttons"; look: ButtonsLook }
  | { kind: "spacer" }
  | { kind: "empty" };

const BUTTON_COUNT = Object.keys(ICONS).length;

function slotWidth(slot: Slot): number {
  switch (slot.kind) {
    case "lights":
      return (
        TRAFFIC_LIGHTS.length * TRAFFIC_LIGHT.size + (TRAFFIC_LIGHTS.length - 1) * TRAFFIC_LIGHT.gap
      );
    case "buttons":
      return BUTTON_COUNT * slot.look.size + (BUTTON_COUNT - 1) * slot.look.gap;
    case "spacer":
    case "empty":
      return 0;
  }
}

type StyleLook = {
  windowRadius: number;
  fieldRadius: number;
  fieldAlign: "center" | "flex-start";
  lead: Slot;
  trail: Slot;
};

const TITLE_BAR = { gap: 10, padX: 14 } as const;

const ADDRESS_FIELD = {
  height: 22,
  fontSize: 11.5,
  padX: 12,
  fontWeight: 450,
  letterSpacing: "0.01em",
} as const;

const STYLE_LOOKS: Record<WindowStyle, StyleLook> = {
  macos: {
    windowRadius: WINDOW_RADIUS,
    fieldRadius: 7,
    fieldAlign: "center",
    lead: { kind: "lights" },
    trail: { kind: "spacer" },
  },
  linux: {
    windowRadius: WINDOW_RADIUS,
    fieldRadius: 7,
    fieldAlign: "center",
    lead: { kind: "spacer" },
    trail: { kind: "buttons", look: { size: 20, gap: 9, glyph: 11, fullHeight: false } },
  },
  windows: {
    windowRadius: 8,
    fieldRadius: PILL_RADIUS,
    fieldAlign: "flex-start",
    lead: { kind: "empty" },
    trail: { kind: "buttons", look: { size: 44, gap: 0, glyph: 12, fullHeight: true } },
  },
};

const WindowButtons: React.FC<{ look: ButtonsLook; theme: FrameTheme }> = ({ look, theme }) => {
  const k = useDesignScale();
  const c = FRAME_COLORS[theme];
  const row: React.CSSProperties = look.fullHeight
    ? { alignSelf: "stretch", marginRight: -TITLE_BAR.padX * k }
    : { gap: look.gap * k };
  const button: React.CSSProperties = look.fullHeight
    ? { width: look.size * k, alignSelf: "stretch" }
    : {
        width: look.size * k,
        height: look.size * k,
        borderRadius: PILL_RADIUS,
        background: c.button,
      };
  return (
    <div style={{ display: "flex", flexShrink: 0, ...row }}>
      {Object.entries(ICONS).map(([name, d]) => (
        <div key={name} style={{ display: "grid", placeItems: "center", ...button }}>
          <Glyph d={d} size={look.glyph * k} color={c.icon} />
        </div>
      ))}
    </div>
  );
};

const SlotView: React.FC<{ slot: Slot; facing: Slot; theme: FrameTheme }> = ({
  slot,
  facing,
  theme,
}) => {
  const k = useDesignScale();
  switch (slot.kind) {
    case "lights":
      return <TrafficLights />;
    case "buttons":
      return <WindowButtons look={slot.look} theme={theme} />;
    case "spacer":
      return <div style={{ width: slotWidth(facing) * k, flexShrink: 0 }} />;
    case "empty":
      return null;
  }
};

const AddressField: React.FC<{ look: StyleLook; theme: FrameTheme; address: string }> = ({
  look,
  theme,
  address,
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
        fontWeight: ADDRESS_FIELD.fontWeight,
        letterSpacing: ADDRESS_FIELD.letterSpacing,
        display: "flex",
        alignItems: "center",
        justifyContent: look.fieldAlign,
        padding: `0 ${ADDRESS_FIELD.padX * k}px`,
        overflow: "hidden",
        whiteSpace: "nowrap",
        textOverflow: "ellipsis",
      }}
    >
      {address}
    </div>
  );
};

const TitleBar: React.FC<{
  windowStyle: WindowStyle;
  theme: FrameTheme;
  addressBar: boolean;
  address: string;
}> = ({ windowStyle, theme, addressBar, address }) => {
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
      <SlotView slot={look.lead} facing={look.trail} theme={theme} />
      {addressBar ? (
        <AddressField look={look} theme={theme} address={address} />
      ) : (
        <div style={{ flex: 1 }} />
      )}
      <SlotView slot={look.trail} facing={look.lead} theme={theme} />
    </div>
  );
};

export const WindowFrame: React.FC<{
  chrome?: boolean;
  address?: string;
  windowStyle?: WindowStyle;
  theme?: FrameTheme;
  addressBar?: boolean;
  children: React.ReactNode;
}> = ({
  chrome = false,
  address = "",
  windowStyle = DEFAULT_WINDOW_STYLE,
  theme = DEFAULT_FRAME_THEME,
  addressBar = DEFAULT_ADDRESS_BAR,
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
          address={address}
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
