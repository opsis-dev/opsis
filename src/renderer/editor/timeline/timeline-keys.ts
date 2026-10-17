import { usePlayback } from "../../store/playback";
import { useTimeline } from "./timeline-state";
import { COMMAND_KEYS, TOOLS, ZOOM_STEP } from "./timeline-tools";
import type { TimelineCommands } from "./use-timeline-commands";

type KeyContext = {
  commands: TimelineCommands;
  fps: number;
  duration: number;
  shift: boolean;
};

type KeyAction = (c: KeyContext) => void;

type Keymap = Partial<Record<string, KeyAction>>;

const playback = () => usePlayback.getState();

const zoomIn: KeyAction = (c) => c.commands.zoomBy(ZOOM_STEP);
const zoomOut: KeyAction = (c) => c.commands.zoomBy(1 / ZOOM_STEP);
const remove: KeyAction = (c) => c.commands.remove();

const ZOOM_KEYS: Keymap = { "=": zoomIn, "+": zoomIn, "-": zoomOut };

const CTRL_KEYS: Keymap = ZOOM_KEYS;

const ALT_KEYS: Keymap = { x: () => playback().clearRange() };

const commandKey = (key: string) => key.toLowerCase();

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
  ...ZOOM_KEYS,
  [commandKey(COMMAND_KEYS.fit)]: (c) => c.commands.fit(),
  [commandKey(COMMAND_KEYS.split)]: (c) => c.commands.split(),
  [commandKey(COMMAND_KEYS.addMarker)]: (c) => c.commands.addMarker(),
  delete: remove,
  backspace: remove,
  ...Object.fromEntries(
    TOOLS.map((t) => [commandKey(t.key), () => useTimeline.getState().setTool(t.id)]),
  ),
};

export function timelineKeyAction(e: React.KeyboardEvent): KeyAction | undefined {
  const key = e.key.toLowerCase();
  if (e.ctrlKey || e.metaKey) return CTRL_KEYS[key];
  return (e.altKey ? ALT_KEYS : PLAIN_KEYS)[key];
}
