import { z } from "zod";
import { workflowSchema, workflowSettingsSchema } from "../schema/workflow";
import {
  addEdgeBody,
  addNodeBody,
  bodySchema,
  openBody,
  runBody,
  updateNodeBody,
  variablesBody,
} from "./bodies";

const ref = (name: string) => ({ $ref: `#/components/schemas/${name}` });

const json = (schema: unknown, description = "OK") => ({
  description,
  content: { "application/json": { schema } },
});

const body = (schema: unknown) => ({
  required: true,
  content: { "application/json": { schema } },
});

const errors = {
  "400": json(ref("Error"), "The request isn't valid"),
  "404": json(ref("Error"), "Not found"),
  "409": json(ref("Error"), "No project is open, or a run is already going"),
};

const projectRead = json({
  type: "object",
  required: ["dir", "workflow", "revision", "saved", "running"],
  properties: {
    dir: { type: "string" },
    workflow: ref("Workflow"),
    revision: { type: "integer", description: "Goes up with every edit" },
    saved: { type: "boolean" },
    running: { type: "boolean" },
  },
});

const edited = json(ref("Edited"));

export function openApiDocument(version: string): Record<string, unknown> {
  return {
    openapi: "3.1.0",
    info: {
      title: "Demoflow local API",
      version,
      description: [
        "Reads, edits and runs the project open in Demoflow. It listens only on a local socket (a named pipe on Windows) that only your user account can open, and only while it's switched on in Settings → API.",
        "Edits go through the open editor: each request is one undo step, saves itself and shows in every window.",
        "Start with GET /nodes for the node types, GET /nodes/{type} for one type's ports and settings, and GET /project for the open project.",
        "A connection joins an output port (source, sourceHandle) to an input port (target, targetHandle). Ports have types; GET /nodes/{type} lists them.",
      ].join("\n\n"),
    },
    servers: [{ url: "http://localhost", description: "Over the local socket or pipe" }],
    paths: {
      "/openapi.json": {
        get: { summary: "This document", responses: { "200": json({ type: "object" }) } },
      },
      "/status": {
        get: {
          summary: "App version, the open project and whether a run is going",
          responses: { "200": json(ref("Status")) },
        },
      },
      "/nodes": {
        get: {
          summary: "Every node type, without its settings schema",
          responses: { "200": json({ type: "array", items: ref("NodeType") }) },
        },
      },
      "/nodes/{type}": {
        get: {
          summary: "One node type with its ports and settings as JSON Schema",
          parameters: [{ name: "type", in: "path", required: true, schema: { type: "string" } }],
          responses: { "200": json(ref("NodeType")), "404": errors["404"] },
        },
      },
      "/validate": {
        post: {
          summary: "Check a project without opening it",
          description:
            "Checks the file format, node types, connections and every node's settings. Nothing is saved.",
          requestBody: body(ref("Workflow")),
          responses: { "200": json(ref("Check")) },
        },
      },
      "/open": {
        post: {
          summary: "Open a project folder in the editor",
          requestBody: body(bodySchema(openBody)),
          responses: { "200": projectRead, "400": errors["400"], "404": errors["404"] },
        },
      },
      "/project": {
        get: {
          summary: "The open project",
          responses: { "200": projectRead, "409": errors["409"] },
        },
        put: {
          summary: "Replace the open project's flow",
          requestBody: body(ref("Workflow")),
          responses: { "200": edited, "400": errors["400"], "409": errors["409"] },
        },
      },
      "/project/nodes": {
        post: {
          summary: "Add a node",
          description: "Settings you leave out get their defaults.",
          requestBody: body(bodySchema(addNodeBody)),
          responses: { "200": edited, "400": errors["400"], "409": errors["409"] },
        },
      },
      "/project/nodes/{id}": {
        patch: {
          summary: "Change a node's name, position, settings or whether it's skipped",
          description: "params is merged into the node's settings.",
          parameters: [{ name: "id", in: "path", required: true, schema: { type: "string" } }],
          requestBody: body(bodySchema(updateNodeBody)),
          responses: { "200": edited, "400": errors["400"], "404": errors["404"] },
        },
        delete: {
          summary: "Delete a node and its connections",
          parameters: [{ name: "id", in: "path", required: true, schema: { type: "string" } }],
          responses: { "200": edited, "404": errors["404"] },
        },
      },
      "/project/edges": {
        post: {
          summary: "Connect an output port to an input port",
          requestBody: body(bodySchema(addEdgeBody)),
          responses: { "200": edited, "400": errors["400"], "404": errors["404"] },
        },
      },
      "/project/edges/{id}": {
        delete: {
          summary: "Remove a connection",
          parameters: [{ name: "id", in: "path", required: true, schema: { type: "string" } }],
          responses: { "200": edited, "404": errors["404"] },
        },
      },
      "/project/settings": {
        patch: {
          summary: "Change the project's size, frame rate, background, theme or fonts",
          requestBody: body(ref("ProjectSettings")),
          responses: { "200": edited, "400": errors["400"], "409": errors["409"] },
        },
      },
      "/project/variables": {
        put: {
          summary: "Replace the project's variables",
          requestBody: body(bodySchema(variablesBody)),
          responses: { "200": edited, "400": errors["400"], "409": errors["409"] },
        },
      },
      "/project/problems": {
        get: {
          summary: "Problems in the open project",
          responses: { "200": json(ref("Check")), "409": errors["409"] },
        },
      },
      "/project/undo": {
        post: { summary: "Undo the last edit", responses: { "200": edited } },
      },
      "/project/redo": {
        post: { summary: "Redo", responses: { "200": edited } },
      },
      "/project/save": {
        post: { summary: "Save now", responses: { "200": edited } },
      },
      "/project/run": {
        get: {
          summary: "Whether a run is going",
          responses: { "200": json(ref("RunStatus")) },
        },
        post: {
          summary: "Start a run",
          description:
            "With no body, runs everything that needs running. targetNodeIds runs those nodes and what they need; fromNodeId runs again from that node. Runs open the websites in the flow.",
          requestBody: {
            required: false,
            content: { "application/json": { schema: bodySchema(runBody) } },
          },
          responses: {
            "200": json(ref("RunStatus")),
            "400": errors["400"],
            "409": errors["409"],
          },
        },
      },
      "/project/run/stop": {
        post: { summary: "Stop the run", responses: { "200": json(ref("RunStatus")) } },
      },
      "/project/run/events": {
        get: {
          summary: "Follow run progress",
          description:
            "A server-sent event stream. Each event is a JSON run event (run:start, node:start, node:done, node:error, node:skipped, run:done, …).",
          responses: { "200": { description: "text/event-stream" } },
        },
      },
    },
    components: {
      schemas: {
        Workflow: z.toJSONSchema(workflowSchema, { io: "input", unrepresentable: "any" }),
        ProjectSettings: z.toJSONSchema(workflowSettingsSchema.partial(), {
          io: "input",
          unrepresentable: "any",
        }),
        Problem: {
          type: "object",
          required: ["level", "message"],
          properties: {
            level: { enum: ["error", "warning"] },
            message: { type: "string" },
            nodeId: { type: "string" },
            edgeId: { type: "string" },
            path: { type: "string", description: "Where in the file, for format problems" },
          },
        },
        Check: {
          type: "object",
          required: ["ok", "problems"],
          properties: {
            ok: { type: "boolean", description: "True when there are no errors" },
            problems: { type: "array", items: ref("Problem") },
          },
        },
        Edited: {
          type: "object",
          required: ["revision", "problems"],
          properties: {
            id: { type: "string", description: "The new node's or connection's id" },
            revision: { type: "integer" },
            problems: { type: "array", items: ref("Problem") },
          },
        },
        Port: {
          type: "object",
          properties: {
            type: { type: "string" },
            label: { type: "string" },
            optional: { type: "boolean" },
            many: { type: "boolean", description: "Takes any number of connections" },
          },
        },
        NodeType: {
          type: "object",
          required: ["type", "title", "category", "inputs", "outputs"],
          properties: {
            type: { type: "string" },
            title: { type: "string" },
            category: { type: "string" },
            description: { type: "string" },
            runs: { type: "boolean", description: "Does work in a run (records, exports)" },
            inputs: { type: "object", additionalProperties: ref("Port") },
            outputs: { type: "object", additionalProperties: ref("Port") },
            params: { type: "object", description: "JSON Schema of the node's settings" },
          },
        },
        RunStatus: {
          type: "object",
          properties: {
            running: { type: "boolean" },
            runId: { type: ["string", "null"] },
          },
        },
        Status: {
          type: "object",
          properties: {
            app: { type: "string" },
            version: { type: "string" },
            project: {
              type: ["object", "null"],
              properties: { dir: { type: "string" }, name: { type: "string" } },
            },
            running: { type: "boolean" },
          },
        },
        Error: {
          type: "object",
          required: ["error"],
          properties: {
            error: { type: "string" },
            problems: { type: "array", items: ref("Problem") },
          },
        },
      },
    },
  };
}
