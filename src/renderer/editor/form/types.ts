import type { FieldMeta } from "@/shared/elements/introspect";
import type { VarInfo } from "../var-text";

export type SpotBackground = { src: string; timeMs: number } | null;

export type FormContext = {
  projectDir: string | null;
  fps: number;
  aspect: number;
  vars: VarInfo[];
  secrets: string[] | null;
  layers: number;
  spotBackground: SpotBackground;
  pickTarget: (key: string) => void;
  chooseAsset: (key: string) => void;
  actions: Array<{ id: string; label: string }>;
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
