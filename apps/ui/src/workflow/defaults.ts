import type { Viewport, WorkflowSettings } from "@/generated/bindings";

export const THEME_BACKGROUNDS: Record<WorkflowSettings["theme"], string> = {
  light: "#fafafa",
  dark: "#09090b",
};

export const DEFAULT_VIEWPORT: Viewport = { width: 1920, height: 1080 };
