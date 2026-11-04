import type { FieldMeta } from "@/editor/form/schema";
import type { MediaKind, PortChoice, TakeKind } from "@/generated/bindings";
import type { VarInfo } from "../var-text";

export type SpotBackground = {
  src: string;
  timeMs: number;
  durationMs: number;
  still?: boolean;
  aspect?: number | null;
} | null;

export type FormContext = {
  nodeId: string;
  projectDir: string | null;
  fps: number;
  aspect: number;
  vars: VarInfo[];
  layers: number;
  spotBackground: SpotBackground;
  pickTarget: (key: string) => void;
  chooseAsset: (apply: (path: string) => void) => void;
  swatches: string[];
  actions: Array<{ id: string; label: string }>;
  markers: Array<{ id: string; label: string }>;
  connectedInputs: string[];
  ports: PortChoice[];
  fileKinds: Partial<Record<string, MediaKind>>;
  takeKind: TakeKind | null;
};

export type ParamValues = Record<string, unknown>;

export type FieldProps<V> = {
  id: string;
  label: string;
  meta: FieldMeta;
  value: V;
  onChange: (value: V) => void;
  ctx: FormContext;
};

export type ParamFieldProps<V> = FieldProps<V> & {
  name: string;
  all: ParamValues;
  onPatch: (patch: ParamValues) => void;
};
