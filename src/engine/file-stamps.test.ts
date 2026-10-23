import { mkdir, utimes, writeFile } from "node:fs/promises";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { fileStamps, signInStamp } from "./file-stamps";
import { tempDirs } from "./test-temp-dirs";

const newDir = tempDirs("opsis-stamps-");

describe("fileStamps", () => {
  it("changes when a project file is replaced in place", async () => {
    const dir = await newDir();
    await mkdir(path.join(dir, "assets"));
    const file = path.join(dir, "assets/clip.mp4");
    await writeFile(file, "first");
    const params = { path: "assets/clip.mp4", label: "Intro" };
    const before = fileStamps(dir, params);
    await writeFile(file, "second take");
    await utimes(file, new Date(), new Date(Date.now() + 5000));
    const after = fileStamps(dir, params);
    expect(Object.keys(before)).toEqual(["assets/clip.mp4"]);
    expect(after).not.toEqual(before);
  });

  it("reads files in rows and project refs, and skips missing files and files outside", async () => {
    const dir = await newDir();
    await writeFile(path.join(dir, "logo.png"), "png");
    const stamps = fileStamps(dir, {
      rows: [{ image: "project:logo.png" }, { image: "gone.png" }],
      outside: "../secret.txt",
      app: "app:placeholders/logo.svg",
    });
    expect(Object.keys(stamps)).toEqual(["project:logo.png"]);
  });
});

describe("signInStamp", () => {
  it("changes when the sign-in is saved again, and is empty before it is saved", async () => {
    const dir = await newDir();
    expect(signInStamp(dir, "acct")).toBeNull();
    await mkdir(path.join(dir, "secrets"));
    const file = path.join(dir, "secrets", "acct.json.enc");
    await writeFile(file, "one");
    const first = signInStamp(dir, "acct");
    await writeFile(file, "a longer one");
    expect(signInStamp(dir, "acct")).not.toBe(first);
  });
});
