import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { KeyframeEdit } from "@/generated/bindings";
import { EnvelopeField } from "./envelope-field";
import type { FormContext } from "./types";

const sent = vi.hoisted(() => [] as KeyframeEdit[]);

vi.mock("@/store/workflow", () => ({
  workflowEdits: { keyframes: (edit: KeyframeEdit) => sent.push(edit) },
}));

const ctx: FormContext = {
  nodeId: "title",
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
  ports: [],
  fileKinds: {},
};

afterEach(cleanup);

describe("the keyframes field", () => {
  it("asks Rust to add and remove keys on its node", () => {
    render(
      <EnvelopeField
        id="e"
        label="Keyframes"
        meta={{}}
        value={{ opacity: [{ frame: 0, value: 1, ease: "linear" }] }}
        onChange={() => undefined}
        ctx={ctx}
        tracks={["opacity", "x"]}
      />,
    );
    expect(screen.getByText("Opacity")).toBeTruthy();
    expect(screen.getByText("Move across")).toBeTruthy();
    fireEvent.click(screen.getAllByRole("button", { name: "Add" })[1]);
    fireEvent.click(screen.getByRole("button", { name: "Remove keyframe" }));
    expect(sent).toEqual([
      { op: "appendKey", nodeId: "title", track: "x" },
      { op: "removeKey", nodeId: "title", track: "opacity", index: 0 },
    ]);
  });
});
