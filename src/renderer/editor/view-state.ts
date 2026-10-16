import { setViewProject } from "../store/view-prefs";
import { storedQuality, useUiStore } from "../store/ui";
import { storedTab, usePanels } from "./panels";
import { storedTimelinePrefs, useTimeline } from "./timeline/timeline-state";

export function loadViewState(projectDir: string | null): void {
  setViewProject(projectDir);
  usePanels.setState({ active: storedTab() });
  useTimeline.setState({ ...storedTimelinePrefs(), fit: true, selectedMarker: null });
  useUiStore.setState({ previewQuality: storedQuality() });
}
