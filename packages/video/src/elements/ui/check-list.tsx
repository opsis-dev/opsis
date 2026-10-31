import { AbsoluteFill } from "@opsis/runtime/parts.tsx";
import { useAuthoredFrame } from "#video/core/authored-clock.ts";
import type { ElementSettings } from "#video/generated/spec.ts";
import { DEFAULT_STEP, randomBetween, steppedRamp } from "#video/lib/motion/stop-motion.ts";
import { HAND_FAMILY, Handwrite, handwriteDuration } from "#video/parts/handwrite.tsx";

function easeOutCubic(t: number) {
  return 1 - (1 - t) ** 3;
}

const DEFAULT_PER_STEP = 1.6;

export type CheckListItem = {
  text: string;
  checked?: boolean;
};

export type CheckListEntry = {
  text: string;
  checked: boolean;
};

export type CheckListGeometry = {
  boxSize: number;
  boxGap: number;
  rowHeight: number;
  labelWidth: number;
};

export type CheckListBeat = {
  boxFrom: number;
  boxTo: number;
  labelAt: number;
  labelEnd: number;
  tickFrom?: number;
  tickTo?: number;
  strikeFrom?: number;
  strikeTo?: number;
};

export type CheckListTiming = {
  delay?: number;
  itemGap?: number;
  perStep?: number;
  step?: number;
  closeGap?: number;
};

export function normalizeCheckListItems(items: (string | CheckListItem)[]): CheckListEntry[] {
  return items.map((item) =>
    typeof item === "string"
      ? { text: item, checked: true }
      : { text: item.text, checked: item.checked ?? true },
  );
}

export function checkListGeometry(width: number, fontSize: number): CheckListGeometry {
  const boxSize = Math.round(fontSize * 0.62);
  const boxGap = Math.round(fontSize * 0.4);
  return {
    boxSize,
    boxGap,
    rowHeight: Math.round(fontSize * 1.15),
    labelWidth: Math.max(fontSize * 2, width - boxSize - boxGap),
  };
}

export function checkListRowWidth(width: number, fontSize: number): number {
  const { boxSize, boxGap, labelWidth } = checkListGeometry(width, fontSize);
  return boxSize + boxGap + labelWidth;
}

export function checkListEnterEnd(
  items: (string | CheckListItem)[],
  options?: CheckListTiming,
): number {
  const step = options?.step ?? DEFAULT_STEP;
  const delay = options?.delay ?? 0;
  const itemGap = options?.itemGap ?? step * 6;
  const perStep = options?.perStep ?? DEFAULT_PER_STEP;

  return normalizeCheckListItems(items).reduce(
    (end, item, i) =>
      Math.max(
        end,
        delay + i * itemGap + step * 2 + handwriteDuration(item.text, { perStep, step }),
      ),
    delay,
  );
}

export function checkListSchedule(
  items: (string | CheckListItem)[],
  options?: CheckListTiming,
): CheckListBeat[] {
  const step = options?.step ?? DEFAULT_STEP;
  const delay = options?.delay ?? 0;
  const itemGap = options?.itemGap ?? step * 6;
  const perStep = options?.perStep ?? DEFAULT_PER_STEP;
  const closeGap = options?.closeGap ?? step * 3;
  const entries = normalizeCheckListItems(items);
  const enterEnd = checkListEnterEnd(entries, options);

  const closeOrder = new Map<number, number>();
  entries.forEach((item, i) => {
    if (item.checked) {
      closeOrder.set(i, closeOrder.size);
    }
  });

  return entries.map((item, i) => {
    const boxFrom = delay + i * itemGap;
    const boxTo = boxFrom + step * 2;
    const labelEnd = boxTo + handwriteDuration(item.text, { perStep, step });
    const slot = closeOrder.get(i);
    if (slot === undefined) {
      return { boxFrom, boxTo, labelAt: boxTo, labelEnd };
    }
    const tickFrom = enterEnd + slot * closeGap;
    const tickTo = tickFrom + step * 2;
    return {
      boxFrom,
      boxTo,
      labelAt: boxTo,
      labelEnd,
      tickFrom,
      tickTo,
      strikeFrom: tickTo,
      strikeTo: tickTo + step * 2,
    };
  });
}

export function checkListDuration(
  items: (string | CheckListItem)[],
  options?: CheckListTiming,
): number {
  return checkListSchedule(items, options).reduce(
    (end, beat) => Math.max(end, beat.strikeTo ?? beat.labelEnd),
    0,
  );
}

function boxPath(size: number, seed: string, index: number, wobble: number): string {
  function nudge(n: number) {
    return randomBetween(`${seed}:${index}:c${n}`, -1.5, 1.5) * wobble;
  }
  const lo = size * 0.08;
  const hi = size - lo;
  const corners: [number, number][] = [
    [lo + nudge(0), lo + nudge(1)],
    [hi + nudge(2), lo + nudge(3)],
    [hi + nudge(4), hi + nudge(5)],
    [lo + nudge(6), hi + nudge(7)],
  ];
  const [start, ...rest] = corners;
  return `M ${start[0]} ${start[1]} ${rest.map(([x, y]) => `L ${x} ${y}`).join(" ")} Z`;
}

function tickPath(size: number): string {
  return `M ${size * 0.2} ${size * 0.52} L ${size * 0.42} ${size * 0.78} L ${size * 0.88} ${size * 0.14}`;
}

const CAVEAT_ADVANCE = 0.36;

export function checkListStrikeWidth(text: string, fontSize: number, labelWidth: number): number {
  return Math.min(labelWidth, Array.from(text).length * fontSize * CAVEAT_ADVANCE);
}

function strikePath(
  reach: number,
  fontSize: number,
  rowHeight: number,
  seed: string,
  index: number,
  wobble: number,
): string {
  function lift(n: number) {
    return randomBetween(`${seed}:${index}:s${n}`, -1.6, 1.6) * wobble;
  }
  const overshoot = fontSize * 0.07;
  const mid = rowHeight / 2;
  return `M ${-overshoot} ${mid + lift(0)} C ${reach * 0.35} ${mid + lift(1)}, ${reach * 0.7} ${mid + lift(2)}, ${reach + overshoot} ${mid + lift(3)}`;
}

export function CheckList({
  background,
  items,
  width,
  fontSize,
  color,
  boxColor,
  tickColor,
  strikeColor,
  delay = 0,
  itemGap,
  closeGap,
  rowGap,
  strokeWidth,
  perStep,
  weight,
  seed,
  step,
  wobble,
  fontFamily,
}: ElementSettings["check-list"] & { delay?: number }) {
  const frame = useAuthoredFrame();
  const entries = normalizeCheckListItems(items);
  const schedule = checkListSchedule(entries, {
    delay,
    itemGap,
    closeGap,
    perStep,
    step,
  });
  const { boxSize, boxGap, rowHeight, labelWidth } = checkListGeometry(width, fontSize);
  function labelSize(text: string) {
    return Math.min(fontSize, labelWidth / Math.max(1, Array.from(text).length * CAVEAT_ADVANCE));
  }

  return (
    <AbsoluteFill
      style={{ alignItems: "center", justifyContent: "center", textAlign: "center", background }}
    >
      <div
        style={{
          width: checkListRowWidth(width, fontSize),
          display: "flex",
          flexDirection: "column",
          gap: rowGap ?? fontSize * 0.55,
        }}
      >
        {entries.map((item, i) => {
          const beat = schedule[i];
          const boxProgress = steppedRamp(frame, beat.boxFrom, beat.boxTo, {
            ease: easeOutCubic,
            step,
          });
          const tickProgress =
            beat.tickFrom === undefined || beat.tickTo === undefined
              ? 0
              : steppedRamp(frame, beat.tickFrom, beat.tickTo, {
                  ease: easeOutCubic,
                  step,
                });
          const strikeProgress =
            beat.strikeFrom === undefined || beat.strikeTo === undefined
              ? 0
              : steppedRamp(frame, beat.strikeFrom, beat.strikeTo, {
                  ease: easeOutCubic,
                  step,
                });

          return (
            <div
              key={`${item.text}:${i}`}
              style={{
                display: "flex",
                alignItems: "center",
                gap: boxGap,
                height: rowHeight,
              }}
            >
              <svg
                width={boxSize}
                height={boxSize}
                viewBox={`0 0 ${boxSize} ${boxSize}`}
                style={{ display: "block", overflow: "visible", flexShrink: 0 }}
              >
                <path
                  d={boxPath(boxSize, seed, i, wobble)}
                  stroke={boxColor}
                  strokeWidth={strokeWidth}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  fill="none"
                  pathLength={1}
                  strokeDasharray={1}
                  strokeDashoffset={1 - boxProgress}
                  opacity={boxProgress > 0 ? 0.9 : 0}
                />
                {item.checked ? (
                  <path
                    d={tickPath(boxSize)}
                    stroke={tickColor}
                    strokeWidth={strokeWidth + 1}
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    fill="none"
                    pathLength={1}
                    strokeDasharray={1}
                    strokeDashoffset={1 - tickProgress}
                    opacity={tickProgress > 0 ? 1 : 0}
                  />
                ) : null}
              </svg>
              <div
                style={{
                  position: "relative",
                  flexShrink: 0,
                  width: labelWidth,
                  height: rowHeight,
                }}
              >
                <Handwrite
                  text={item.text}
                  fontSize={labelSize(item.text)}
                  color={color}
                  delay={beat.labelAt}
                  perStep={perStep}
                  weight={weight}
                  align="left"
                  step={step}
                  wobble={wobble}
                  fontFamily={fontFamily || HAND_FAMILY}
                />
                {item.checked ? (
                  <svg
                    width={labelWidth}
                    height={rowHeight}
                    viewBox={`0 0 ${labelWidth} ${rowHeight}`}
                    style={{
                      position: "absolute",
                      inset: 0,
                      display: "block",
                      overflow: "visible",
                    }}
                  >
                    <path
                      d={strikePath(
                        checkListStrikeWidth(item.text, labelSize(item.text), labelWidth),
                        labelSize(item.text),
                        rowHeight,
                        seed,
                        i,
                        wobble,
                      )}
                      stroke={strikeColor || tickColor}
                      strokeWidth={strokeWidth}
                      strokeLinecap="round"
                      fill="none"
                      pathLength={1}
                      strokeDasharray={1}
                      strokeDashoffset={1 - strikeProgress}
                      opacity={strikeProgress > 0 ? 0.85 : 0}
                    />
                  </svg>
                ) : null}
              </div>
            </div>
          );
        })}
      </div>
    </AbsoluteFill>
  );
}
