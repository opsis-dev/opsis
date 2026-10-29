import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { windowDragHandlers } from "./window-drag";

const calls = vi.hoisted(() => [] as string[]);

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  getCurrentWebviewWindow: () => ({
    startDragging: () => Promise.resolve(void calls.push("drag")),
  }),
}));

vi.mock("@/generated/bindings", () => ({
  commands: { windowRole: (role: string) => Promise.resolve(void calls.push(role)) },
}));

afterEach(() => {
  cleanup();
  calls.length = 0;
});

function Bar() {
  return (
    <header data-testid="bar" {...windowDragHandlers}>
      <span>Title</span>
      <button type="button">Menu</button>
    </header>
  );
}

describe("dragging the window by its app bar", () => {
  it("starts dragging only once the pointer moves", () => {
    render(<Bar />);
    fireEvent.mouseDown(screen.getByText("Title"), {
      button: 0,
      detail: 1,
      clientX: 10,
      clientY: 10,
    });
    expect(calls).toEqual([]);
    fireEvent.mouseMove(window, { clientX: 11, clientY: 10 });
    expect(calls).toEqual([]);
    fireEvent.mouseMove(window, { clientX: 20, clientY: 10 });
    fireEvent.mouseMove(window, { clientX: 40, clientY: 10 });
    expect(calls).toEqual(["drag"]);
  });

  it("maximizes on a double-click without dragging", () => {
    render(<Bar />);
    const title = screen.getByText("Title");
    fireEvent.mouseDown(title, { button: 0, detail: 1, clientX: 10, clientY: 10 });
    fireEvent.mouseUp(window);
    fireEvent.mouseDown(title, { button: 0, detail: 2, clientX: 10, clientY: 10 });
    fireEvent.mouseMove(window, { clientX: 30, clientY: 10 });
    fireEvent.doubleClick(title, { button: 0 });
    expect(calls).toEqual(["toggleMaximize"]);
  });

  it("leaves the controls in the bar alone", () => {
    render(<Bar />);
    const button = screen.getByRole("button", { name: "Menu" });
    fireEvent.mouseDown(button, { button: 0, detail: 1, clientX: 10, clientY: 10 });
    fireEvent.mouseMove(window, { clientX: 40, clientY: 10 });
    fireEvent.doubleClick(button, { button: 0 });
    expect(calls).toEqual([]);
  });
});
