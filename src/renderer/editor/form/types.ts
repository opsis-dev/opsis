import type { FieldMeta } from "@/shared/elements/introspect";
import type { VarInfo } from "../var-text";

export type SpotBackground = { src: string; timeMs: number } | null;

export type FormContext = {
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
  connectedInputs: string[];
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
