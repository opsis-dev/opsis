import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type * as Bindings from "@/generated/bindings";
import type { GradientEdit, GradientView } from "@/generated/bindings";
import { GradientField } from "./gradient-field";
import type { FormContext } from "./types";

const rust = vi.hoisted(() => {
  const view: GradientView = {
    gradient: {
      kind: "linear",
      angle: 90,
      stops: [
        { color: "#000000", at: 0 },
        { color: "#ff0000", at: 50 },
        { color: "#ffffff", at: 100 },
      ],
    },
    css: "linear-gradient(90deg, #000000 0%, #ff0000 50%, #ffffff 100%)",
    strip: "linear-gradient(90deg, #000000 0%, #ff0000 50%, #ffffff 100%)",
    canAdd: true,
    canRemove: true,
  };
  return { view, edits: [] as Array<{ css: string; edit: GradientEdit }> };
});

vi.mock("@/generated/bindings", async (original) => {
  const bindings = await original<typeof Bindings>();
  return {
    ...bindings,
    commands: {
      ...bindings.commands,
      gradientView: () => Promise.resolve(rust.view),
      editGradient: (css: string, edit: GradientEdit) => {
        rust.edits.push({ css, edit });
        return Promise.resolve({ ...rust.view, css: "edited", selected: 0 });
      },
    },
  };
});

const ctx: FormContext = {
  nodeId: "n",
  projectDir: null,
  fps: 30,
  aspect: 16 / 9,
  vars: [],
  layers: 1,
  spotBackground: null,
  pickTarget: () => undefined,
  chooseAsset: () => undefined,
  swatches: [],
  actions: [],
  connectedInputs: [],
  fileKinds: {},
};

afterEach(cleanup);

describe("the gradient field", () => {
  it("draws the stops Rust parsed and sends each edit to Rust", async () => {
    const changes: string[] = [];
    render(
      <GradientField
        id="g"
        label="Gradient"
        meta={{}}
        value={rust.view.css}
        onChange={(css) => changes.push(css)}
        ctx={ctx}
      />,
    );
    expect(await screen.findByLabelText("Colour stop 2 at 50%")).toBeTruthy();
    fireEvent.click(screen.getByLabelText("Remove this colour stop"));
    await vi.waitFor(() => expect(changes).toEqual(["edited"]));
    expect(rust.edits).toEqual([{ css: rust.view.css, edit: { op: "removeStop", index: 0 } }]);
  });
});
