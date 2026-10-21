import { describe, expect, it } from "vitest";
import { cursorKeyword } from "./cursor-keyword";

const keyword = (cursor: string, editable = false) => cursorKeyword({ cursor, editable });

describe("the cursor a page asks for", () => {
  it("keeps only the keyword after a custom image, so nothing loads from the page's site", () => {
    expect(keyword("url(a.png) 4 4, pointer")).toBe("pointer");
    expect(keyword('url("data:image/svg+xml,<svg a=1,b=2>") 2 3, text')).toBe("text");
    expect(keyword("url(a.cur), url(b.png) 1 1, Grab")).toBe("grab");
  });

  it("shows a text cursor for auto over a field and the arrow anywhere else", () => {
    expect(keyword("auto", true)).toBe("text");
    expect(keyword("auto")).toBe("default");
  });

  it("falls back to the arrow when no keyword is left", () => {
    expect(keyword("url(a.png)")).toBe("default");
    expect(keyword("")).toBe("default");
  });

  it("passes plain keywords through", () => {
    expect(keyword("pointer", true)).toBe("pointer");
    expect(keyword("not-allowed")).toBe("not-allowed");
  });
});
