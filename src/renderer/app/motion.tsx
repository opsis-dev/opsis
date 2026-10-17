import {
  AnimatePresence,
  MotionConfig,
  motion,
  useSpring,
  useTransform,
  type Variants,
} from "motion/react";
import { useEffect, type ReactNode } from "react";
import { cn } from "@/renderer/lib/utils";
import { percentText } from "@/shared/format";

const EASE_OUT = [0.22, 1, 0.36, 1] as const;

export function MotionProvider({ children }: { children: ReactNode }) {
  return (
    <MotionConfig reducedMotion="user" transition={{ duration: 0.22, ease: EASE_OUT }}>
      {children}
    </MotionConfig>
  );
}

export function AnimatedPercent({ value, className }: { value: number; className?: string }) {
  const spring = useSpring(value, { stiffness: 180, damping: 28 });
  const text = useTransform(spring, (v) => percentText(Math.round(v) / 100));
  useEffect(() => spring.set(value), [spring, value]);
  return <motion.span className={cn("tabular-nums", className)}>{text}</motion.span>;
}

export function SwapFade({
  id,
  children,
  className,
}: {
  id: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <AnimatePresence mode="wait" initial={false}>
      <motion.div
        key={id}
        className={className}
        initial={{ opacity: 0, y: 6 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, transition: { duration: 0.08 } }}
        transition={{ duration: 0.18, ease: EASE_OUT }}
      >
        {children}
      </motion.div>
    </AnimatePresence>
  );
}

const group: Variants = { hidden: {}, shown: { transition: { staggerChildren: 0.04 } } };
const member: Variants = {
  hidden: { opacity: 0, y: 8, filter: "blur(4px)" },
  shown: { opacity: 1, y: 0, filter: "blur(0px)", transition: { duration: 0.3, ease: EASE_OUT } },
};

export function Stagger({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <motion.div className={className} variants={group} initial="hidden" animate="shown">
      {children}
    </motion.div>
  );
}

export function StaggerItem({ children, className }: { children: ReactNode; className?: string }) {
  return (
    <motion.div className={className} variants={member}>
      {children}
    </motion.div>
  );
}

export function Ripple({
  cx,
  cy,
  from,
  to,
  opacity,
  strokeWidth,
  seconds,
  className,
}: {
  cx: number;
  cy: number;
  from: number;
  to: number;
  opacity: number;
  strokeWidth: number;
  seconds: number;
  className?: string;
}) {
  return (
    <motion.circle
      cx={cx}
      cy={cy}
      className={className}
      strokeWidth={strokeWidth}
      initial={{ r: from, opacity }}
      animate={{ r: to, opacity: 0 }}
      transition={{ duration: seconds, ease: "linear" }}
    />
  );
}
