import { usePlayback } from "../../store/playback";
import { TOOLS, ZOOM_STEP } from "./timeline-toolbar";
import { useTimeline } from "./timeline-state";
import type { TimelineActions } from "./timeline-view";

type KeyContext = {
  actions: TimelineActions | null;
  fps: number;
  duration: number;
  shift: boolean;
};

type KeyAction = (c: KeyContext) => void;

type Keymap = Partial<Record<string, KeyAction>>;

const playback = () => usePlayback.getState();

const zoomIn: KeyAction = (c) => c.actions?.zoomBy(ZOOM_STEP);
const zoomOut: KeyAction = (c) => c.actions?.zoomBy(1 / ZOOM_STEP);
const remove: KeyAction = (c) => c.actions?.remove();

const CTRL_KEYS: Keymap = { "=": zoomIn, "+": zoomIn, "-": zoomOut };

const ALT_KEYS: Keymap = { x: () => playback().clearRange() };

const PLAIN_KEYS: Keymap = {
  " ": () => playback().toggle(),
  j: () => playback().shuttle(-1),
  k: () => playback().shuttle(0),
  l: () => playback().shuttle(1),
  arrowleft: (c) => playback().step(c.shift ? -c.fps : -1),
  arrowright: (c) => playback().step(c.shift ? c.fps : 1),
  home: () => playback().seek(0),
  end: (c) => playback().seek(c.duration - 1),
  i: () => playback().setIn(playback().frame),
  o: () => playback().setOut(playback().frame),
  "=": zoomIn,
  "+": zoomIn,
  "-": zoomOut,
  z: (c) => c.actions?.fit(),
  s: (c) => c.actions?.split(),
  m: (c) => c.actions?.addMarker(),
  delete: remove,
  backspace: remove,
  ...Object.fromEntries(
    TOOLS.map((t) => [t.key.toLowerCase(), () => useTimeline.getState().setTool(t.id)]),
  ),
};

export function timelineKeyAction(e: React.KeyboardEvent): KeyAction | undefined {
  const key = e.key.toLowerCase();
  if (e.ctrlKey || e.metaKey) return CTRL_KEYS[key];
  return (e.altKey ? ALT_KEYS : PLAIN_KEYS)[key];
}
