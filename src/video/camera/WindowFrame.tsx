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

export const WindowFrame: React.FC<{
  chrome?: boolean;
  hostLabel?: string;
  children: React.ReactNode;
}> = ({ chrome = false, hostLabel = "", children }) => {
  const k = useDesignScale();
  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        borderRadius: WINDOW_RADIUS * k,
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
            background: "linear-gradient(180deg, #fafbfc 0%, #f0f2f5 100%)",
            borderBottom: "1px solid rgba(15, 23, 42, 0.07)",
            boxShadow: "inset 0 -1px 0 rgba(255,255,255,0.6)",
          }}
        >
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
          <div
            style={{
              flex: 1,
              height: 22 * k,
              borderRadius: 7 * k,
              background: "rgba(15, 23, 42, 0.045)",
              border: `${k}px solid rgba(15, 23, 42, 0.05)`,
              color: "rgba(15, 23, 42, 0.42)",
              fontSize: 11.5 * k,
              fontFamily: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif',
              fontWeight: 450,
              letterSpacing: "0.01em",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: `0 ${12 * k}px`,
              overflow: "hidden",
              whiteSpace: "nowrap",
              textOverflow: "ellipsis",
            }}
          >
            {hostLabel}
          </div>
          <div style={{ width: 52 * k, flexShrink: 0 }} />
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
