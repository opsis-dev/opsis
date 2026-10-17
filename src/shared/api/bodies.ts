import { z } from "zod";

const OBJECT_BODY = { error: "The request body must be a JSON object." };
const POSITION = "position needs numbers x and y.";

const text = (name: string) => z.string({ error: `${name} must be text.` });
const required = (name: string) => z.string({ error: `${name} is required.` });

const positionSchema = z.object(
  { x: z.number({ error: POSITION }), y: z.number({ error: POSITION }) },
  { error: POSITION },
);

const paramsSchema = z.record(z.string(), z.unknown(), { error: "params must be an object." });

export const objectBody = z.record(z.string(), z.unknown(), OBJECT_BODY);

export const openBody = z.object(
  {
    dir: z
      .string({ error: "dir must be an absolute folder path." })
      .meta({ description: "The project folder" }),
  },
  OBJECT_BODY,
);

export const addNodeBody = z.object(
  {
    type: required("type"),
    id: text("id").meta({ description: "Leave out to get a new id" }).optional(),
    label: text("label").optional(),
    position: positionSchema.optional(),
    params: paramsSchema.optional(),
  },
  OBJECT_BODY,
);

export const updateNodeBody = z.object(
  {
    label: text("label").nullable().optional(),
    position: positionSchema.optional(),
    disabled: z.boolean({ error: "disabled must be true or false." }).optional(),
    params: paramsSchema.optional(),
  },
  OBJECT_BODY,
);

export const addEdgeBody = z.object(
  {
    source: required("source"),
    sourceHandle: required("sourceHandle"),
    target: required("target"),
    targetHandle: required("targetHandle"),
  },
  OBJECT_BODY,
);

export const variablesBody = z.record(
  z.string(),
  z.string({ error: "Every variable must be text." }),
  OBJECT_BODY,
);

export const runBody = z.object(
  {
    targetNodeIds: z
      .array(text("Every node id"), { error: "targetNodeIds must be a list of node ids." })
      .optional(),
    fromNodeId: text("fromNodeId").optional(),
  },
  OBJECT_BODY,
);

export const JSON_SCHEMA_OPTIONS = { io: "input", unrepresentable: "any" } as const;

export const jsonSchemaOf = (schema: z.ZodType) => z.toJSONSchema(schema, JSON_SCHEMA_OPTIONS);
