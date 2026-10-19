import { z } from "zod";
import { workflowSchema, workflowSettingsSchema } from "../schema/workflow";
import {
  addEdgeBody,
  addNodeBody,
  JSON_SCHEMA_OPTIONS,
  jsonSchemaOf,
  openBody,
  runBody,
  updateNodeBody,
  variablesBody,
} from "./bodies";
import { catalogNodeSchema, catalogPortSchema } from "./catalog";
import {
  apiCheckSchema,
  apiEditedSchema,
  apiErrorSchema,
  apiProblemSchema,
  apiProjectStateSchema,
  apiRunStateSchema,
  apiStatusSchema,
} from "./types";

const SCHEMAS = "#/components/schemas/";

const components = z.registry<{ id: string }>();
components.add(workflowSchema, { id: "Workflow" });
components.add(workflowSettingsSchema.partial(), { id: "ProjectSettings" });
components.add(apiProblemSchema, { id: "Problem" });
components.add(apiCheckSchema, { id: "Check" });
components.add(apiEditedSchema, { id: "Edited" });
components.add(apiProjectStateSchema, { id: "Project" });
components.add(apiRunStateSchema, { id: "RunStatus" });
components.add(apiStatusSchema, { id: "Status" });
components.add(catalogPortSchema, { id: "Port" });
components.add(catalogNodeSchema, { id: "NodeType" });
components.add(apiErrorSchema, { id: "Error" });

type JsonSchema = z.core.JSONSchema.JSONSchema;

type JsonContent = { "application/json": { schema: JsonSchema } };

type ApiResponse = { description: string; content?: JsonContent };

type PathParameter = { name: string; in: "path"; required: true; schema: JsonSchema };

type Operation = {
  summary: string;
  description?: string;
  parameters?: PathParameter[];
  requestBody?: { required: boolean; content: JsonContent };
  responses: Record<string, ApiResponse>;
};

type PathItem = Partial<Record<"get" | "post" | "put" | "patch" | "delete", Operation>>;

const ref = (name: string): JsonSchema => ({ $ref: `${SCHEMAS}${name}` });

const content = (schema: JsonSchema): JsonContent => ({ "application/json": { schema } });

const json = (schema: JsonSchema, description = "OK"): ApiResponse => ({
  description,
  content: content(schema),
});

const body = (schema: JsonSchema, required = true) => ({ required, content: content(schema) });

const errors = {
  "400": json(ref("Error"), "The request isn't valid"),
  "404": json(ref("Error"), "Not found"),
  "409": json(ref("Error"), "No project is open, or a run is already going"),
};

const projectRead = json(ref("Project"));

const edited = json(ref("Edited"));

const pathParam = (name: string): PathParameter[] => [
  { name, in: "path", required: true, schema: { type: "string" } },
];

const projectOp = (op: Operation): Operation => ({
  ...op,
  responses: { ...op.responses, "409": errors["409"] },
});

const DESCRIPTION = [
  "Reads, edits and runs the project open in Opsis. It listens only on a local socket (a named pipe on Windows) that only your user account can open, and only while it's switched on in Settings → API.",
  "Edits go through the open editor: each request is one undo step, saves itself and shows in every window.",
  "Start with GET /nodes for the node types, GET /nodes/{type} for one type's ports and settings, and GET /project for the open project.",
  "A connection joins an output port (source, sourceHandle) to an input port (target, targetHandle). Ports have types; GET /nodes/{type} lists them.",
].join("\n\n");

const PATHS: Record<string, PathItem> = {
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
      parameters: pathParam("type"),
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
      requestBody: body(jsonSchemaOf(openBody)),
      responses: { "200": projectRead, "400": errors["400"], "404": errors["404"] },
    },
  },
  "/project": {
    get: projectOp({
      summary: "The open project",
      responses: { "200": projectRead },
    }),
    put: projectOp({
      summary: "Replace the open project's flow",
      requestBody: body(ref("Workflow")),
      responses: { "200": edited, "400": errors["400"] },
    }),
  },
  "/project/nodes": {
    post: projectOp({
      summary: "Add a node",
      description: "Settings you leave out get their defaults.",
      requestBody: body(jsonSchemaOf(addNodeBody)),
      responses: { "200": edited, "400": errors["400"] },
    }),
  },
  "/project/nodes/{id}": {
    patch: projectOp({
      summary: "Change a node's name, position, settings or whether it's skipped",
      description: "params is merged into the node's settings.",
      parameters: pathParam("id"),
      requestBody: body(jsonSchemaOf(updateNodeBody)),
      responses: { "200": edited, "400": errors["400"], "404": errors["404"] },
    }),
    delete: projectOp({
      summary: "Delete a node and its connections",
      parameters: pathParam("id"),
      responses: { "200": edited, "404": errors["404"] },
    }),
  },
  "/project/edges": {
    post: projectOp({
      summary: "Connect an output port to an input port",
      requestBody: body(jsonSchemaOf(addEdgeBody)),
      responses: { "200": edited, "400": errors["400"], "404": errors["404"] },
    }),
  },
  "/project/edges/{id}": {
    delete: projectOp({
      summary: "Remove a connection",
      parameters: pathParam("id"),
      responses: { "200": edited, "404": errors["404"] },
    }),
  },
  "/project/settings": {
    patch: projectOp({
      summary: "Change the project's size, frame rate, background, theme or fonts",
      requestBody: body(ref("ProjectSettings")),
      responses: { "200": edited, "400": errors["400"] },
    }),
  },
  "/project/variables": {
    put: projectOp({
      summary: "Replace the project's variables",
      requestBody: body(jsonSchemaOf(variablesBody)),
      responses: { "200": edited, "400": errors["400"] },
    }),
  },
  "/project/problems": {
    get: projectOp({
      summary: "Problems in the open project",
      responses: { "200": json(ref("Check")) },
    }),
  },
  "/project/undo": {
    post: projectOp({ summary: "Undo the last edit", responses: { "200": edited } }),
  },
  "/project/redo": {
    post: projectOp({ summary: "Redo", responses: { "200": edited } }),
  },
  "/project/save": {
    post: projectOp({ summary: "Save now", responses: { "200": edited } }),
  },
  "/project/run": {
    get: projectOp({
      summary: "Whether a run is going",
      responses: { "200": json(ref("RunStatus")) },
    }),
    post: projectOp({
      summary: "Start a run",
      description:
        "With no body, runs everything that needs running. targetNodeIds runs those nodes and what they need; fromNodeId runs again from that node. Runs open the websites in the flow.",
      requestBody: body(jsonSchemaOf(runBody), false),
      responses: {
        "200": json(ref("RunStatus")),
        "400": errors["400"],
      },
    }),
  },
  "/project/run/stop": {
    post: projectOp({ summary: "Stop the run", responses: { "200": json(ref("RunStatus")) } }),
  },
  "/project/run/events": {
    get: {
      summary: "Follow run progress",
      description:
        "A server-sent event stream. Each event is a JSON run event (run:start, node:start, node:done, node:error, node:skipped, run:done, …).",
      responses: { "200": { description: "text/event-stream" } },
    },
  },
};

const componentSchemas = () =>
  z.toJSONSchema(components, { ...JSON_SCHEMA_OPTIONS, uri: (id) => `${SCHEMAS}${id}` }).schemas;

export function openApiDocument(version: string) {
  return {
    openapi: "3.1.0",
    info: { title: "Opsis local API", version, description: DESCRIPTION },
    servers: [{ url: "http://localhost", description: "Over the local socket or pipe" }],
    paths: PATHS,
    components: { schemas: componentSchemas() },
  };
}
