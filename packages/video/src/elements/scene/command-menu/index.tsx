import { Easing } from "@opsis/runtime/easing.ts";
import { interpolate } from "@opsis/runtime/interpolate.ts";
import { fontOr } from "#video/common/fonts.ts";
import { themeColors } from "#video/common/theme.ts";
import { CLAMPED } from "#video/core/anim.ts";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import {
  type CommandMenuIcon,
  CommandMenuItemRow,
  type CommandMenuItemState,
  type CommandMenuItemStyleContext,
  commandMenuItemStyle,
  commandMenuItemStyleContext,
} from "#video/elements/scene/command-menu/command-menu-item.tsx";
import type { ElementSettings } from "#video/generated/spec.ts";
import { type NeutralTheme, useNeutralTheme } from "#video/lib/neutral-ui/index.ts";
import {
  NeutralUIProvider,
  defaultDarkTheme,
  defaultLightTheme,
} from "#video/lib/neutral-ui/theme.ts";
import { revealedText } from "#video/lib/ui-kit/timeline.ts";

export type CommandMenuState = "opened" | "closed";

export interface CommandMenuEntry {
  icon?: CommandMenuIcon;
  label: string;
  shortcut?: string;
}

type CommandMenuViewProps = Omit<ElementSettings["command-menu"], "theme"> & {
  theme?: Partial<NeutralTheme>;
};

const OPEN_AT = 6;
const OPEN_FOR = 12;
const TYPE_AT = 24;
const HOVER_AFTER_TYPING = 8;
const PRESS_FOR = 6;

export function commandMenuPlayEnd(query: string, perLetter: number, pause: number): number {
  return TYPE_AT + Array.from(query).length * perLetter + HOVER_AFTER_TYPING + pause + PRESS_FOR;
}

function blendStyles(a: CommandMenuStyle, b: CommandMenuStyle, t: number): CommandMenuStyle {
  return {
    backdropOpacity: a.backdropOpacity + (b.backdropOpacity - a.backdropOpacity) * t,
    panelOpacity: a.panelOpacity + (b.panelOpacity - a.panelOpacity) * t,
    panelScale: a.panelScale + (b.panelScale - a.panelScale) * t,
    panelTranslateY: a.panelTranslateY + (b.panelTranslateY - a.panelTranslateY) * t,
  };
}

export function filterCommandItems(
  items: CommandMenuEntry[],
  query: string,
  revealCount?: number,
): CommandMenuEntry[] {
  const visible = (revealCount === undefined ? query : revealedText(query, revealCount))
    .trim()
    .toLowerCase();
  if (visible === "") return items;
  return items.filter((item) => item.label.toLowerCase().includes(visible));
}

export interface CommandMenuStyle {
  backdropOpacity: number;
  panelOpacity: number;
  panelScale: number;
  panelTranslateY: number;
}

export interface CommandMenuStyleContext {
  panelBg: string;
  panelBorder: string;
  inputFg: string;
  placeholderFg: string;
  mutedFg: string;
  divider: string;
  caret: string;
  radius: number;
  itemCtx: CommandMenuItemStyleContext;
}

export function commandMenuStyleContext(theme: NeutralTheme): CommandMenuStyleContext {
  return {
    panelBg: theme.popover,
    panelBorder: theme.border,
    inputFg: theme.popoverForeground,
    placeholderFg: theme.mutedForeground,
    mutedFg: theme.mutedForeground,
    divider: theme.border,
    caret: theme.foreground,
    radius: theme.radius,
    itemCtx: commandMenuItemStyleContext(theme),
  };
}

export function commandMenuStyle(
  state: CommandMenuState,
  _ctx: CommandMenuStyleContext,
): CommandMenuStyle {
  switch (state) {
    case "opened":
      return {
        backdropOpacity: 1,
        panelOpacity: 1,
        panelScale: 1,
        panelTranslateY: 0,
      };
    default:
      return {
        backdropOpacity: 0,
        panelOpacity: 0,
        panelScale: 0.96,
        panelTranslateY: 8,
      };
  }
}

function rowState(
  i: number,
  selectedIndex: number,
  highlightedIndex: number,
  pressedIndex: number,
): CommandMenuItemState {
  if (i === pressedIndex) return "press";
  if (i === selectedIndex) return "selected";
  if (i === highlightedIndex) return "hover";
  return "idle";
}

function CommandMenuView({
  state,
  query,
  revealCount,
  items,
  selectedIndex,
  highlightedIndex,
  pressedIndex,
  theme: themeOverride,
  autoplay,
  letterSeconds: perLetter,
  pauseSeconds: pause,
  speed,
  pickIndex,
  placeholder,
  emptyText,
  backdropOpacity,
  panelWidth,
  top,
  fontFamily,
}: CommandMenuViewProps) {
  const frame = useAuthoredFrame() * speed;
  const theme = useNeutralTheme(themeOverride, "light");
  const ctx = commandMenuStyleContext(theme);
  const opened = interpolate(frame, [OPEN_AT, OPEN_AT + OPEN_FOR], [0, 1], {
    ...CLAMPED,
    easing: Easing.out(Easing.cubic),
  });
  const typed = Math.max(0, Math.floor((frame - TYPE_AT) / perLetter) + 1);
  const shownCount = autoplay ? Math.min(typed, Array.from(query).length) : revealCount;
  const v = autoplay
    ? blendStyles(commandMenuStyle("closed", ctx), commandMenuStyle("opened", ctx), opened)
    : commandMenuStyle(state, ctx);

  const visibleQuery = shownCount === undefined ? query : revealedText(query, shownCount);
  const filtered = filterCommandItems(items, query, shownCount);
  const typingEnd = TYPE_AT + Array.from(query).length * perLetter;
  const pick = items.indexOf(filtered[Math.min(pickIndex, filtered.length - 1)]);
  const hoverAt = typingEnd + HOVER_AFTER_TYPING;
  const pressAt = hoverAt + pause;
  const highlighted = autoplay ? (frame >= hoverAt ? pick : -1) : highlightedIndex;
  const pressed = autoplay
    ? frame >= pressAt && frame < pressAt + PRESS_FOR
      ? pick
      : -1
    : pressedIndex;
  const selected = autoplay ? (frame >= pressAt + PRESS_FOR ? pick : -1) : selectedIndex;

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "center",
        paddingTop: `${top}%`,
        fontFamily: fontOr(fontFamily, "main"),
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: 0,
          background: `rgba(0, 0, 0, ${backdropOpacity * v.backdropOpacity})`,
        }}
      />
      <div
        style={{
          position: "relative",
          width: panelWidth,
          boxSizing: "border-box",
          transformOrigin: "top",
          transform: `translateY(${v.panelTranslateY}px) scale(${v.panelScale})`,
          opacity: v.panelOpacity,
          background: ctx.panelBg,
          border: `1px solid ${ctx.panelBorder}`,
          borderRadius: ctx.radius + 6,
          padding: 8,
          display: "flex",
          flexDirection: "column",
          boxShadow: "0 24px 48px -12px rgba(0,0,0,0.25)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: 10,
            padding: "8px 10px",
          }}
        >
          <svg width={18} height={18} viewBox="0 0 24 24" fill="none">
            <path
              d="M11 4a7 7 0 1 0 0 14 7 7 0 0 0 0-14ZM20 20l-3.5-3.5"
              stroke={ctx.mutedFg}
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
          <span
            style={{
              display: "flex",
              alignItems: "center",
              fontSize: 15,
              letterSpacing: "-0.01em",
              color: visibleQuery ? ctx.inputFg : ctx.placeholderFg,
            }}
          >
            {visibleQuery || placeholder}
            <span
              style={{
                display: "inline-block",
                width: 1.5,
                height: 18,
                marginLeft: 1,
                background: ctx.caret,
              }}
            />
          </span>
        </div>
        <div
          style={{
            height: 1,
            background: ctx.divider,
            margin: "4px 0",
          }}
        />
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: 2,
            padding: "4px 0",
          }}
        >
          {filtered.length === 0 ? (
            <div
              style={{
                padding: "20px 12px",
                textAlign: "center",
                fontSize: 14,
                color: ctx.mutedFg,
              }}
            >
              {emptyText}
            </div>
          ) : (
            filtered.map((item) => {
              const index = items.indexOf(item);
              return (
                <CommandMenuItemRow
                  key={index}
                  style={commandMenuItemStyle(
                    rowState(index, selected, highlighted, pressed),
                    ctx.itemCtx,
                  )}
                  ctx={ctx.itemCtx}
                  label={item.label}
                  icon={item.icon}
                  shortcut={item.shortcut}
                  width={panelWidth - 16}
                  radius={theme.radius}
                />
              );
            })
          )}
        </div>
      </div>
    </div>
  );
}

export function CommandMenu({ theme, radius, ...settings }: ElementSettings["command-menu"]) {
  const tokens = {
    ...(theme.mode === "dark" ? defaultDarkTheme : defaultLightTheme),
    ...themeColors(theme),
    radius,
  };
  return (
    <NeutralUIProvider mode={theme.mode} theme={tokens}>
      <CommandMenuView {...settings} radius={radius} theme={tokens} />
    </NeutralUIProvider>
  );
}
