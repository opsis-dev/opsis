import { request } from "node:http";
import { API_SOCKET } from "./paths";

type Answer = { status: number; body: unknown };

export function api(method: string, route: string, body?: unknown): Promise<Answer> {
  return new Promise((resolve, reject) => {
    const sent = body === undefined ? undefined : JSON.stringify(body);
    const req = request(
      { socketPath: API_SOCKET, method, path: route, headers: { host: "localhost" } },
      (res) => {
        let text = "";
        res.setEncoding("utf8");
        res.on("data", (chunk: string) => (text += chunk));
        res.on("end", () =>
          resolve({ status: res.statusCode ?? 0, body: text ? JSON.parse(text) : null }),
        );
      },
    );
    req.on("error", reject);
    req.end(sent);
  });
}

export function runEvents(onEvent: (event: Record<string, unknown>) => boolean): Promise<void> {
  return new Promise((resolve, reject) => {
    const req = request(
      {
        socketPath: API_SOCKET,
        method: "GET",
        path: "/project/run/events",
        headers: { host: "localhost" },
      },
      (res) => {
        let buffered = "";
        res.setEncoding("utf8");
        res.on("data", (chunk: string) => {
          buffered += chunk;
          const blocks = buffered.split("\n\n");
          buffered = blocks.pop() ?? "";
          for (const block of blocks) {
            const data = block.split("\n").find((line) => line.startsWith("data: "));
            if (data && onEvent(JSON.parse(data.slice(6)) as Record<string, unknown>)) {
              req.destroy();
              resolve();
            }
          }
        });
      },
    );
    req.on("error", reject);
    req.end();
  });
}
