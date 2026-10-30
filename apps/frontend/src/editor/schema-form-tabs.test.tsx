import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import * as fc from "fast-check";
import { afterEach, describe, expect, it } from "vitest";
import type { FormContext } from "./form/types";
import { SchemaForm } from "./schema-form";

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
  markers: [],
  connectedInputs: [],
  ports: [],
  fileKinds: {},
};

function schemaOf(fields: Array<{ label: string; advanced: boolean }>) {
  return {
    type: "object",
    properties: Object.fromEntries(
      fields.map(({ label, advanced }, i) => [
        `field${i}`,
        {
          type: "number",
          description: label,
          default: i,
          ...(advanced ? { "x-advanced": true } : {}),
        },
      ]),
    ),
  };
}

function shown(label: string) {
  return screen.queryByText(label) !== null;
}

const fields = fc
  .uniqueArray(fc.stringMatching(/^[A-Z][a-z]{3,8}$/), { minLength: 1, maxLength: 8 })
  .chain((labels) =>
    fc.tuple(
      fc.constant(labels),
      fc.array(fc.boolean(), { minLength: labels.length, maxLength: labels.length }),
    ),
  )
  .map(([labels, flags]) => labels.map((label, i) => ({ label, advanced: flags[i] })));

describe("settings tabs", () => {
  afterEach(cleanup);

  it("opens on Basic, counts the advanced settings and shows each group on its own tab", () => {
    fc.assert(
      fc.property(fields, (list) => {
        render(
          <SchemaForm
            tabs
            schema={schemaOf(list)}
            value={{}}
            onChange={() => undefined}
            ctx={ctx}
          />,
        );
        const basic = list.filter((f) => !f.advanced);
        const advanced = list.filter((f) => f.advanced);
        expect(screen.queryByRole("tablist") !== null).toBe(advanced.length > 0);
        expect(basic.every((f) => shown(f.label))).toBe(true);
        const advancedTab = screen.queryByRole("tab", { name: /Advanced/ });
        expect(advancedTab?.textContent ?? `Advanced${advanced.length}`).toBe(
          `Advanced${advanced.length}`,
        );
        expect(advanced.some((f) => shown(f.label))).toBe(
          advanced.length > 0 && advancedTab === null,
        );
        if (advancedTab) fireEvent.click(advancedTab);
        expect(advanced.every((f) => shown(f.label))).toBe(true);
        cleanup();
      }),
      { numRuns: 40 },
    );
  });

  it("lists every setting in order in a nested form", () => {
    const list = [
      { label: "Width", advanced: false },
      { label: "Softness", advanced: true },
    ];
    render(<SchemaForm schema={schemaOf(list)} value={{}} onChange={() => undefined} ctx={ctx} />);
    expect(screen.queryByRole("tablist")).toBeNull();
    expect(list.every((f) => shown(f.label))).toBe(true);
  });
});
