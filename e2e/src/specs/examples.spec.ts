import path from "node:path";
import { $, $$, browser, expect } from "@wdio/globals";
import { api } from "../support/api";
import { PROJECTS } from "../support/paths";

const EXAMPLES = [
  "todomvc",
  "feature-launch",
  "changelog",
  "onboarding",
  "social-teaser",
  "before-after",
];

type Project = { workflow: { nodes: unknown[] } };

describe("examples", () => {
  for (const example of EXAMPLES)
    it(`opens ${example} with every node on the canvas and no problems`, async () => {
      const opened = await api("POST", "/open", { dir: path.join(PROJECTS, example) });
      expect(opened.status).toBe(200);
      const { workflow } = opened.body as Project;
      await browser.waitUntil(
        async () => (await $$(".react-flow__node").length) === workflow.nodes.length,
        { timeoutMsg: `the canvas never showed the ${workflow.nodes.length} nodes of ${example}` },
      );
      await expect($("//*[normalize-space()='Something went wrong']")).not.toBeDisplayed();
      const problems = await api("GET", "/project/problems");
      expect(problems.body).toMatchObject({ ok: true });
    });
});
