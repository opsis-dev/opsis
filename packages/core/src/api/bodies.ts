import { z } from "zod";

const OBJECT_BODY = { error: "The request body must be a JSON object." };
const text = () => z.string({ error: "must be text." });
const required = () => z.string({ error: "is required." });
const coordinate = () => z.number({ error: "must be a number." });

const positionSchema = z.object(
  { x: coordinate(), y: coordinate() },
  { error: "needs numbers x and y." },
);

const paramsSchema = z.record(z.string(), z.unknown(), { error: "must be an object." });

export const objectBody = z.record(z.string(), z.unknown(), OBJECT_BODY);

export const openBody = z.object(
  {
    dir: z
      .string({ error: "must be an absolute folder path." })
      .meta({ description: "The project folder" }),
  },
  OBJECT_BODY,
);

export const addNodeBody = z.object(
  {
    type: required(),
    id: text().meta({ description: "Leave out to get a new id" }).optional(),
    label: text().optional(),
    position: positionSchema.optional(),
    params: paramsSchema.optional(),
  },
  OBJECT_BODY,
);

export const updateNodeBody = z.object(
  {
    label: text().nullable().optional(),
    position: positionSchema.optional(),
    disabled: z.boolean({ error: "must be true or false." }).optional(),
    params: paramsSchema.optional(),
  },
  OBJECT_BODY,
);

export const addEdgeBody = z.object(
  {
    source: required(),
    sourceHandle: required(),
    target: required(),
    targetHandle: required(),
  },
  OBJECT_BODY,
);

export const variablesBody = z.record(z.string(), text(), OBJECT_BODY);

export const runBody = z.object(
  {
    targetNodeIds: z.array(text(), { error: "must be a list of node ids." }).optional(),
    fromNodeId: text().optional(),
  },
  OBJECT_BODY,
);

export const JSON_SCHEMA_OPTIONS = { io: "input", unrepresentable: "any" } as const;

export const jsonSchemaOf = (schema: z.ZodType) => z.toJSONSchema(schema, JSON_SCHEMA_OPTIONS);
