import { storedViewOptions, useViewOptions } from "../store/view-options";
import type { ViewPrefs } from "../store/view-prefs";
import { setViewProject } from "../store/view-prefs";
import { storedTab, usePanels } from "./panels";
import { useTimeline } from "./timeline/timeline-state";

export function loadViewState(projectDir: string | null, stored: ViewPrefs): void {
  setViewProject(projectDir, stored);
  usePanels.setState({ active: storedTab() });
  useTimeline.getState().resetForProject();
  useViewOptions.setState(storedViewOptions());
}
