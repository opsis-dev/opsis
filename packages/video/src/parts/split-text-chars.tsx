import type { CSSProperties, ReactNode } from "react";
import { useVideoConfig } from "remotion";
import { scaleFont } from "#video/lib/layout/layout.ts";
import {
  useSplitText,
  type SplitMode,
  type SplitOrder,
  type SplitTimingOptions,
  type SplitUnitState,
} from "#video/lib/text/text-split.ts";

export type SplitTextEffect = "fade-up" | "fade" | "scale" | "blur" | "none";

export type SplitTextCharsProps = Omit<SplitTimingOptions, "mode"> & {
  text: string;
  mode?: SplitMode;
  order?: SplitOrder;
  effect?: SplitTextEffect;
  renderUnit?: (unit: SplitUnitState) => ReactNode;
  travel?: number;
  fontSize?: number;
  color?: string;
  fontWeight?: number | string;
  fontFamily?: string;
  lineHeight?: number;
  letterSpacing?: string;
  textAlign?: CSSProperties["textAlign"];
  wordSpacing?: number;
  style?: CSSProperties;
  className?: string;
};

function unitStyle(effect: SplitTextEffect, unit: SplitUnitState, travel: number): CSSProperties {
  const base: CSSProperties = {
    display: "inline-block",
    whiteSpace: "pre",
    willChange: "transform, opacity",
  };

  if (effect === "none") {
    return base;
  }

  if (effect === "fade") {
    return { ...base, opacity: unit.opacity };
  }

  if (effect === "scale") {
    return {
      ...base,
      opacity: unit.opacity,
      scale: 0.72 + unit.value * 0.28,
      transformOrigin: "center bottom",
    };
  }

  if (effect === "blur") {
    const blur = unit.displace * 8;
    return {
      ...base,
      opacity: unit.opacity,
      filter: blur > 0.1 ? `blur(${blur.toFixed(2)}px)` : undefined,
    };
  }

  return {
    ...base,
    opacity: unit.opacity,
    translate: `0 ${(unit.displace * travel).toFixed(4)}em`,
  };
}

export const SplitTextChars: React.FC<SplitTextCharsProps> = ({
  text,
  mode = "chars",
  order = "start",
  effect = "fade-up",
  renderUnit,
  travel = 0.42,
  fontSize: fontSizeProp,
  color = "#f4f4f5",
  fontWeight = 600,
  fontFamily,
  lineHeight = 1.15,
  letterSpacing,
  textAlign = "center",
  wordSpacing = 0.26,
  style,
  className,
  ...timing
}) => {
  const { width } = useVideoConfig();
  const fontSize = fontSizeProp ?? scaleFont(84, width);
  const { lines } = useSplitText({ text, mode, order, ...timing });

  return (
    <span
      className={className}
      style={{
        display: "inline-flex",
        flexDirection: "column",
        alignItems:
          textAlign === "left" ? "flex-start" : textAlign === "right" ? "flex-end" : "center",
        fontSize,
        fontWeight,
        color,
        lineHeight,
        letterSpacing,
        textAlign,
        ...(fontFamily ? { fontFamily } : {}),
        ...style,
      }}
    >
      {lines.map((line) => (
        <span
          key={line.index}
          style={{
            display: "block",
            whiteSpace: "normal",
          }}
        >
          {line.words.map((word, wordIndex) => (
            <span
              key={word.index}
              style={{
                display: "inline-block",
                marginRight: wordIndex === line.words.length - 1 ? 0 : `${wordSpacing}em`,
              }}
            >
              {word.units.map((unit) => (
                <span key={`${unit.index}-${unit.text}`} style={unitStyle(effect, unit, travel)}>
                  {renderUnit ? renderUnit(unit) : unit.text}
                </span>
              ))}
            </span>
          ))}
        </span>
      ))}
    </span>
  );
};
