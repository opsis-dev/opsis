import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { useDecimalInput } from "./decimal";

afterEach(cleanup);

function DecimalField({ apply }: { apply: (text: string) => void }) {
  const input = useDecimalInput("1", "blur", apply);
  return <input aria-label="Seconds" {...input} />;
}

describe("useDecimalInput", () => {
  it("keeps a number typed into a field that closes before it loses focus", () => {
    const applied: string[] = [];
    const view = render(<DecimalField apply={(text) => applied.push(text)} />);
    fireEvent.change(view.getByLabelText("Seconds"), { target: { value: "2.5" } });
    view.unmount();
    expect(applied).toEqual(["2.5"]);
  });
});
