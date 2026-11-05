import { isNumber, isRecord, isString } from "@opsis/video/common/guards";
import type { InternalNode, NodeHandle } from "@xyflow/react";
import { readPref, writePref } from "../store/view-prefs";

const KEY = "cardShapes";
const SAVE_AFTER_MS = 1000;

export type SavedCardShape = { width: number; height: number; handles: NodeHandle[] };

let shapes = new Map<string, SavedCardShape>();
let kept: ReadonlySet<string> = new Set();
let pending: ReturnType<typeof setTimeout> | undefined;

function isHandle(v: unknown): v is NodeHandle {
  return (
    isRecord(v) &&
    (v.type === "source" || v.type === "target") &&
    isString(v.position) &&
    isNumber(v.x) &&
    isNumber(v.y) &&
    (v.id === undefined || v.id === null || isString(v.id))
  );
}

function isShape(v: unknown): v is SavedCardShape {
  return (
    isRecord(v) &&
    isNumber(v.width) &&
    isNumber(v.height) &&
    Array.isArray(v.handles) &&
    v.handles.every(isHandle)
  );
}

function isShapes(v: unknown): v is Record<string, SavedCardShape> {
  return isRecord(v) && Object.values(v).every(isShape);
}

function save(): void {
  pending = undefined;
  for (const id of shapes.keys()) if (!kept.has(id)) shapes.delete(id);
  writePref(KEY, Object.fromEntries(shapes));
}

export function saveCardShapesNow(): void {
  if (pending === undefined) return;
  clearTimeout(pending);
  save();
}

export function loadCardShapes(): void {
  shapes = new Map(Object.entries(readPref(KEY, {}, isShapes)));
}

export function cardShape(id: string): SavedCardShape | undefined {
  return shapes.get(id);
}

function shapeOf(node: InternalNode): SavedCardShape | undefined {
  const { width, height } = node.measured;
  const bounds = node.internals.handleBounds;
  if (!width || !height || !bounds) return undefined;
  const handles = [...(bounds.source ?? []), ...(bounds.target ?? [])].map((h): NodeHandle => ({
    id: h.id,
    type: h.type,
    position: h.position,
    x: h.x,
    y: h.y,
    width: h.width,
    height: h.height,
  }));
  return { width, height, handles };
}

export function rememberCardShapes(
  nodes: readonly InternalNode[],
  nodeIds: ReadonlySet<string>,
): void {
  for (const node of nodes) {
    const shape = shapeOf(node);
    if (shape) shapes.set(node.id, shape);
  }
  kept = nodeIds;
  clearTimeout(pending);
  pending = setTimeout(save, SAVE_AFTER_MS);
}
