import { createServer, type Server } from "node:http";

const HOME_PAGE = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Profile site</title>
<style>
body { font: 18px system-ui, sans-serif; margin: 0; padding: 48px; background: #f6f7f9; color: #111; }
main { max-width: 960px; margin: 0 auto; display: grid; gap: 24px; }
section { background: #fff; border-radius: 12px; padding: 24px; box-shadow: 0 1px 3px #0002; display: grid; gap: 12px; }
button, input, select { font: inherit; padding: 8px 12px; }
#card { width: 160px; padding: 16px; background: #dbeafe; border-radius: 8px; cursor: grab; }
#drop { min-height: 120px; border: 2px dashed #94a3b8; border-radius: 8px; padding: 16px; }
#long { height: 2400px; background: linear-gradient(#fff, #e2e8f0); border-radius: 12px; padding: 24px; }
#banner { padding: 12px; background: #fde68a; border-radius: 8px; }
</style>
</head>
<body>
<main>
<h1 id="title">Profile site</h1>
<div id="banner">This banner is hidden in the take</div>
<section>
<label for="name">Name</label>
<input id="name" placeholder="Your name">
<button id="continue" onclick="document.getElementById('status').textContent = 'Saved ' + document.getElementById('name').value">Continue</button>
<p id="status">Not saved</p>
</section>
<section>
<label for="size">Size</label>
<select id="size"><option value="small">Small</option><option value="large">Large</option></select>
<label for="attachment">Attachment</label>
<input id="attachment" type="file">
</section>
<section>
<div id="card" draggable="true">Card</div>
<div id="drop">Drop here</div>
</section>
<section>
<p id="clock"></p>
<p id="api">Waiting for the API</p>
<a id="next" href="/next">Next page</a>
</section>
<div id="long">Long content</div>
<p id="footer">Footer text</p>
</main>
<script>
document.getElementById("clock").textContent = new Date().toISOString();
const drop = document.getElementById("drop");
drop.addEventListener("dragover", (event) => event.preventDefault());
drop.addEventListener("drop", (event) => { event.preventDefault(); drop.append(document.getElementById("card")); });
fetch("/api/state").then((r) => r.json()).then((body) => { document.getElementById("api").textContent = "API says " + JSON.stringify(body); });
</script>
</body>
</html>`;

const NEXT_PAGE = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Next page</title></head>
<body style="font: 18px system-ui; padding: 48px"><h1 id="arrived">Next page</h1><a href="/">Back</a></body></html>`;

const LOGIN_PAGE = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>Sign in</title></head>
<body style="font: 18px system-ui; padding: 48px">
<h1>Sign in</h1><p>Signing in by itself.</p>
<script>setTimeout(() => { document.cookie = "signed=1; path=/"; location.href = "/"; }, 800);</script>
</body></html>`;

const PAGES: Record<string, string> = { "/": HOME_PAGE, "/next": NEXT_PAGE, "/login": LOGIN_PAGE };

export function serveSite(port: number): Promise<Server> {
  const server = createServer((request, response) => {
    const route = new URL(request.url ?? "/", "http://127.0.0.1").pathname;
    if (route.startsWith("/api/")) {
      response.writeHead(200, { "content-type": "application/json" });
      response.end(JSON.stringify({ state: "real" }));
      return;
    }
    const page = PAGES[route];
    response.writeHead(page ? 200 : 404, { "content-type": "text/html; charset=utf-8" });
    response.end(page ?? "Not found");
  });
  return new Promise((resolve) => server.listen(port, "127.0.0.1", () => resolve(server)));
}
