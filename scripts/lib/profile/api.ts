import { type IncomingMessage, request } from "node:http";

export type RunEvent = {
  type: string;
  runId?: string;
  nodeId?: string;
  cached?: boolean;
  message?: string;
  ok?: boolean;
  failedNodeIds?: string[];
  at?: number;
};

function textOf(response: IncomingMessage): Promise<string> {
  return new Promise((resolve, reject) => {
    let text = "";
    response.setEncoding("utf8");
    response.on("data", (chunk: string) => {
      text += chunk;
    });
    response.on("end", () => resolve(text));
    response.on("error", reject);
  });
}

export function eventsIn(buffer: string): { events: RunEvent[]; rest: string } {
  const blocks = buffer.split("\n\n");
  const rest = blocks.pop() ?? "";
  const events = blocks
    .map((block) =>
      block
        .split("\n")
        .filter((line) => line.startsWith("data: "))
        .map((line) => line.slice("data: ".length))
        .join("\n"),
    )
    .filter(Boolean)
    .map((data) => JSON.parse(data));
  return { events, rest };
}

export function apiClient(socket: string) {
  function call(method: string, route: string, body?: unknown): Promise<unknown> {
    const payload = body === undefined ? undefined : JSON.stringify(body);
    return new Promise((resolve, reject) => {
      const sent = request(
        {
          socketPath: socket,
          path: route,
          method,
          headers: payload === undefined ? {} : { "content-type": "application/json" },
        },
        (response) => {
          textOf(response).then((text) => {
            if ((response.statusCode ?? 0) >= 400)
              reject(new Error(`${method} ${route}: ${response.statusCode} ${text}`));
            else resolve(text ? JSON.parse(text) : null);
          }, reject);
        },
      );
      sent.on("error", reject);
      sent.end(payload);
    });
  }

  function followEvents(onEvent: (event: RunEvent, at: number) => void): Promise<() => void> {
    return new Promise((resolve, reject) => {
      const sent = request({ socketPath: socket, path: "/project/run/events" }, (response) => {
        let pending = "";
        response.setEncoding("utf8");
        response.on("data", (chunk: string) => {
          const { events, rest } = eventsIn(pending + chunk);
          pending = rest;
          for (const event of events) onEvent(event, Date.now());
        });
        resolve(() => sent.destroy());
      });
      sent.on("error", reject);
      sent.end();
    });
  }

  return {
    get(route: string) {
      return call("GET", route);
    },
    post(route: string, body?: unknown) {
      return call("POST", route, body ?? {});
    },
    followEvents,
  };
}

export type ApiClient = ReturnType<typeof apiClient>;
