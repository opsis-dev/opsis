import { setViewProject } from "../store/view-prefs";
import { storedViewOptions, useViewOptions } from "../store/view-options";
import { storedTab, usePanels } from "./panels";
import { useTimeline } from "./timeline/timeline-state";

export function loadViewState(projectDir: string | null): void {
  setViewProject(projectDir);
  usePanels.setState({ active: storedTab() });
  useTimeline.getState().resetForProject();
  useViewOptions.setState(storedViewOptions());
}
