import { type SpringConfig } from "@opsis/runtime/spring.ts";

export const springSmooth: SpringConfig = {
  damping: 200,
  mass: 1,
  stiffness: 100,
  overshootClamping: true,
};

export const springSnappy: SpringConfig = {
  damping: 20,
  mass: 0.8,
  stiffness: 200,
  overshootClamping: true,
};

export const springBouncy: SpringConfig = {
  damping: 12,
  mass: 0.9,
  stiffness: 180,
  overshootClamping: false,
};
