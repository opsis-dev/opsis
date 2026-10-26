import { $, expect } from "@wdio/globals";

describe("start screen", () => {
  it("shows the start actions and the templates from Rust", async () => {
    await expect($("//*[normalize-space()='New project…']")).toBeDisplayed();
    await expect($("//*[normalize-space()='Changelog clip']")).toBeDisplayed();
    await expect($("//*[normalize-space()='Sample: to-do app']")).toBeDisplayed();
  });
});
