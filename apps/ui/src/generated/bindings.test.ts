import type { nodeCatalog } from "@opsis/core/api/catalog";
import type { CompileResult, PreviewTarget } from "@opsis/core/compile/compile";
import type { GraphIssue } from "@opsis/core/graph";
import type { MixPlan } from "@opsis/core/mix/plan";
import type { CardRow } from "@opsis/core/nodes/card";
import type { RunEvent } from "@opsis/core/schema/events";
import type { TimelineSpec } from "@opsis/core/schema/timeline";
import type { CompileValue, LastRun } from "@opsis/core/schema/values";
import type { Workflow } from "@opsis/core/schema/workflow";
import type { FilmChapter } from "@opsis/core/timeline/chapters";
import { describe, expect, it } from "vitest";
import type * as Rust from "./bindings";

type Same<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;

type Fits<A, B> = [A] extends [B] ? true : false;

const SAME: [
  Same<Rust.Workflow, Workflow>,
  Same<Rust.TimelineSpec, TimelineSpec>,
  Same<Rust.CompileValue, CompileValue>,
  Same<Rust.LastRun, LastRun>,
  Same<Rust.RunEvent, RunEvent>,
  Same<Rust.CardRow, CardRow>,
  Fits<Rust.CompileResult, CompileResult>,
  Same<Rust.PreviewTarget, PreviewTarget>,
  Same<Rust.GraphIssue, GraphIssue>,
  Same<Rust.MixPlan, MixPlan>,
  Same<Rust.FilmChapter, FilmChapter>,
  Fits<Rust.CatalogNode, ReturnType<typeof nodeCatalog>[number]>,
] = [true, true, true, true, true, true, true, true, true, true, true, true];

describe("types generated from Rust", () => {
  it("match the shapes the editor reads today", () => {
    expect(SAME.every(Boolean)).toBe(true);
  });
});
