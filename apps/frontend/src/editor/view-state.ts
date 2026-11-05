import { storedViewOptions, useViewOptions } from "../store/view-options";
import type { ViewPrefs } from "../store/view-prefs";
import { setViewProject } from "../store/view-prefs";
import { loadCardShapes, saveCardShapesNow } from "./card-shapes";
import { storedTab, usePanels } from "./panels";
import { useTimeline } from "./timeline/timeline-state";

export function loadViewState(projectDir: string | null, stored: ViewPrefs): void {
  saveCardShapesNow();
  setViewProject(projectDir, stored);
  loadCardShapes();
  usePanels.setState({ active: storedTab() });
  useTimeline.getState().resetForProject();
  useViewOptions.setState(storedViewOptions());
}
