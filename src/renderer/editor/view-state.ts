import { setViewProject } from "../store/view-prefs";
import { storedViewOptions, useViewOptions } from "../store/view-options";
import { storedTab, usePanels } from "./panels";
import { storedTimelinePrefs, useTimeline } from "./timeline/timeline-state";

export function loadViewState(projectDir: string | null): void {
  setViewProject(projectDir);
  usePanels.setState({ active: storedTab() });
  useTimeline.setState({ ...storedTimelinePrefs(), fit: true, selectedMarker: null });
  useViewOptions.setState(storedViewOptions());
}
