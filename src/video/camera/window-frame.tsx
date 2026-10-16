import React from "react";
import { AbsoluteFill, useVideoConfig } from "remotion";
import { DESIGN_WIDTH } from "./lib/click-log";
import { CHROME_H, WINDOW_RADIUS } from "./lib/window";

export const useDesignScale = (): number => {
  const override = React.useContext(DesignWidthContext);
  const width = useVideoConfig().width;
  return (override ?? width) / DESIGN_WIDTH;
};

export const DesignWidthContext = React.createContext<number | null>(null);

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

export const RimLight: React.FC<{ light?: boolean }> = ({ light = false }) => {
  const k = useDesignScale();
  if (light)
    return (
      <>
        <div
          style={{
            position: "absolute",
            inset: 0,
            borderRadius: 60 * k,
            background: "rgba(30,20,10,0.30)",
            transform: "translateY(2.5%) scale(0.97, 0.96)",
            filter: `blur(${90 * k}px)`,
            pointerEvents: "none",
          }}
        />
        <div
          style={{
            position: "absolute",
            inset: -k,
            borderRadius: (WINDOW_RADIUS + 1) * k,
            boxShadow: `0 0 0 ${k}px rgba(15,23,42,0.16)`,
            pointerEvents: "none",
          }}
        />
      </>
    );
  return (
    <>
      <div
        style={{
          position: "absolute",
          inset: 0,
          borderRadius: 60 * k,
          background: "rgba(255,255,255,0.10)",
          transform: "translateY(2%) scale(0.97, 0.96)",
          filter: `blur(${90 * k}px)`,
          pointerEvents: "none",
        }}
      />
      <div
        style={{
          position: "absolute",
          inset: -k,
          borderRadius: (WINDOW_RADIUS + 1) * k,
          boxShadow: `0 0 0 ${k}px rgba(255,255,255,0.14)`,
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

const TrafficLights: React.FC<{ k: number }> = ({ k }) => (
  <div style={{ display: "flex", gap: 7 * k, flexShrink: 0 }}>
    {(
      [
        ["#ff5f57", "#e0443e"],
        ["#febc2e", "#dea123"],
        ["#28c840", "#1aab29"],
      ] as const
    ).map(([fill, rim], i) => (
      <div
        key={i}
        style={{
          width: 11 * k,
          height: 11 * k,
          borderRadius: 99,
          background: `radial-gradient(circle at 35% 30%, ${fill} 0%, ${rim} 100%)`,
          boxShadow: `inset 0 0 0 ${0.5 * k}px rgba(0,0,0,0.12)`,
        }}
      />
    ))}
  </div>
);

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

const WindowButtons: React.FC<{ style: WindowStyle; k: number; theme: FrameTheme }> = ({
  style,
  k,
  theme,
}) => {
  const c = FRAME_COLORS[theme];
  const names = ["minimize", "maximize", "close"] as const;
  if (style === "linux")
    return (
      <div style={{ display: "flex", gap: 9 * k, flexShrink: 0 }}>
        {names.map((n) => (
          <div
            key={n}
            style={{
              width: 20 * k,
              height: 20 * k,
              borderRadius: 99,
              background: c.button,
              display: "grid",
              placeItems: "center",
            }}
          >
            <Glyph d={ICONS[n]} size={11 * k} color={c.icon} />
          </div>
        ))}
      </div>
    );
  return (
    <div style={{ display: "flex", alignSelf: "stretch", flexShrink: 0, marginRight: -14 * k }}>
      {names.map((n) => (
        <div
          key={n}
          style={{ width: 44 * k, display: "grid", placeItems: "center", alignSelf: "stretch" }}
        >
          <Glyph d={ICONS[n]} size={12 * k} color={c.icon} />
        </div>
      ))}
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
  const c = FRAME_COLORS[theme];
  const field = addressBar ? (
    <div
      style={{
        flex: 1,
        height: 22 * k,
        borderRadius: windowStyle === "windows" ? 99 : 7 * k,
        background: c.field,
        border: `${k}px solid ${c.fieldEdge}`,
        color: c.text,
        fontSize: 11.5 * k,
        fontFamily: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif',
        fontWeight: 450,
        letterSpacing: "0.01em",
        display: "flex",
        alignItems: "center",
        justifyContent: windowStyle === "windows" ? "flex-start" : "center",
        padding: `0 ${12 * k}px`,
        overflow: "hidden",
        whiteSpace: "nowrap",
        textOverflow: "ellipsis",
      }}
    >
      {hostLabel}
    </div>
  ) : (
    <div style={{ flex: 1 }} />
  );
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        borderRadius: (windowStyle === "windows" ? 8 : WINDOW_RADIUS) * k,
        overflow: "hidden",
        background: "#fff",
        display: "flex",
        flexDirection: "column",
        filter: "saturate(1.05) contrast(1.025) brightness(1.01)",
        boxShadow: `inset 0 ${k}px 0 rgba(255,255,255,0.9)`,
      }}
    >
      {chrome ? (
        <div
          style={{
            flex: `0 0 ${CHROME_H * k}px`,
            display: "flex",
            alignItems: "center",
            gap: 10 * k,
            padding: `0 ${14 * k}px`,
            background: c.bar,
            borderBottom: `1px solid ${c.edge}`,
          }}
        >
          {windowStyle === "macos" ? <TrafficLights k={k} /> : null}
          {windowStyle === "linux" ? <div style={{ width: 88 * k, flexShrink: 0 }} /> : null}
          {field}
          {windowStyle === "macos" ? (
            <div style={{ width: 52 * k, flexShrink: 0 }} />
          ) : (
            <WindowButtons style={windowStyle} k={k} theme={theme} />
          )}
        </div>
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
