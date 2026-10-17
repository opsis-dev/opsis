import { describe, expect, it } from "vitest";
import { secretFileName, secretName } from "./secrets";

describe("sign-in names", () => {
  it("trims and falls back to default", () => {
    expect(secretName(" work ")).toBe("work");
    expect(secretName("")).toBe("default");
    expect(secretName(undefined)).toBe("default");
  });

  it("gives the same file name to the saving and the reading side", () => {
    expect(secretFileName(" work ")).toBe(secretFileName("work"));
    expect(secretFileName("")).toBe("default");
    expect(secretFileName("my account")).toBe("my-account");
    expect(secretFileName(" Login! ")).toBe("Login");
    expect(secretFileName("!!!")).toBe("default");
  });
});
