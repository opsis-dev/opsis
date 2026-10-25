import { percentText } from "@opsis/core/format";
import {
  AnimatePresence,
  MotionConfig,
  motion,
  useSpring,
  useTransform,
  type Variants,
} from "motion/react";
import { useEffect, type ComponentProps, type ReactNode } from "react";
import { cn } from "@/lib/utils";

const EASE_OUT = [0.22, 1, 0.36, 1] as const;

export function MotionProvider({ children }: { children: ReactNode }) {
  return (
    <MotionConfig reducedMotion="user" transition={{ duration: 0.22, ease: EASE_OUT }}>
      {children}
    </MotionConfig>
  );
}

export function AnimatedPercent({ fraction, className }: { fraction: number; className?: string }) {
  const spring = useSpring(fraction, { stiffness: 180, damping: 28 });
  const text = useTransform(spring, (v) => percentText(v));
  useEffect(() => spring.set(fraction), [spring, fraction]);
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

type RippleCircle = Omit<
  ComponentProps<typeof motion.circle>,
  "r" | "opacity" | "initial" | "animate" | "transition"
>;

export function Ripple({
  from,
  to,
  opacity,
  seconds,
  ...circle
}: RippleCircle & { from: number; to: number; opacity: number; seconds: number }) {
  return (
    <motion.circle
      {...circle}
      initial={{ r: from, opacity }}
      animate={{ r: to, opacity: 0 }}
      transition={{ duration: seconds, ease: "linear" }}
    />
  );
}
