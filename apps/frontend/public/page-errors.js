(() => {
  function send(message) {
    return window.__TAURI_INTERNALS__
      ?.invoke("page_log", { level: "error", message })
      .catch(() => {});
  }
  function describe(value) {
    return value instanceof Error ? value.stack || String(value) : String(value);
  }
  window.addEventListener("error", (e) => send(describe(e.error ?? e.message)));
  window.addEventListener("unhandledrejection", (e) =>
    send(`Unhandled rejection: ${describe(e.reason)}`),
  );
})();
