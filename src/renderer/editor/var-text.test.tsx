import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { VarText } from "./var-text";

vi.hoisted(() => Object.assign(window, { opsis: { platform: "linux" } }));

describe("VarText", () => {
  afterEach(cleanup);

  it("stores an emptied field as empty, not the line break the browser leaves", () => {
    const onChange = vi.fn();
    render(<VarText id="t" value="A" onChange={onChange} vars={[]} />);
    const box = screen.getByRole("textbox");
    box.innerHTML = "<br>";
    fireEvent.input(box);
    expect(onChange).toHaveBeenLastCalledWith("");
  });

  it("keeps a line break the user typed", () => {
    const onChange = vi.fn();
    render(<VarText id="t" value="" onChange={onChange} vars={[]} multiline />);
    const box = screen.getByRole("textbox");
    box.innerHTML = "a<br><br>";
    fireEvent.input(box);
    expect(onChange).toHaveBeenLastCalledWith("a\n");
  });
});
