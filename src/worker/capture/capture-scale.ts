export type Size = { width: number; height: number };

export const DEFAULT_CAPTURE_SCALE = 1;

export const MAX_CAPTURE_SCALE = 4;

export function resolveCaptureScale(raw?: string): number {
  const n = raw != null && raw !== "" ? Number(raw) : DEFAULT_CAPTURE_SCALE;
  if (!Number.isFinite(n) || n <= 1) return 1;
  return Math.min(n, MAX_CAPTURE_SCALE);
}

export function scaledViewport(logical: Size, scale: number): Size {
  const even = (n: number) => Math.round((n * scale) / 2) * 2;
  return { width: even(logical.width), height: even(logical.height) };
}

export function captureScaleInitScript(scale: number, logical: Size): string {
  return `
    (() => {
      var LW = ${logical.width}, LH = ${logical.height}, Z = ${scale};
      try {
        Object.defineProperty(window, "innerWidth",  { get: function () { return LW; }, configurable: true });
        Object.defineProperty(window, "innerHeight", { get: function () { return LH; }, configurable: true });
      } catch (e) {}
      try {
        if (window.visualViewport) {
          Object.defineProperty(window.visualViewport, "width",  { get: function () { return LW; }, configurable: true });
          Object.defineProperty(window.visualViewport, "height", { get: function () { return LH; }, configurable: true });
        }
      } catch (e) {}
      var UNIT = /([-+]?[0-9]*\\.?[0-9]+)(dvh|svh|lvh|vh|dvw|svw|lvw|vw)\\b/g;
      var toPx = function (text) {
        return text.replace(UNIT, function (_, n, unit) {
          var base = unit.charAt(unit.length - 1) === "h" ? LH : LW;
          return (parseFloat(n) / 100) * base + "px";
        });
      };
      var seen = 0;
      var rewrite = function () {
        var sheets = document.styleSheets;
        if (sheets.length === seen) return;
        seen = sheets.length;
        for (var i = 0; i < sheets.length; i++) {
          var rules;
          try { rules = sheets[i].cssRules; } catch (e) { continue; }
          if (!rules) continue;
          for (var j = 0; j < rules.length; j++) {
            var st = rules[j].style;
            if (!st) continue;
            for (var k = 0; k < st.length; k++) {
              var prop = st[k];
              var val = st.getPropertyValue(prop);
              if (val.indexOf("v") === -1) continue;
              var next = toPx(val);
              if (next !== val) st.setProperty(prop, next, st.getPropertyPriority(prop));
            }
          }
        }
      };
      var apply = function () {
        var d = document.documentElement;
        if (d) d.style.zoom = String(Z);
        try { rewrite(); } catch (e) {}
      };
      apply();
      document.addEventListener("DOMContentLoaded", apply);
      window.addEventListener("load", apply);
      var n = 0;
      var tick = function () { apply(); if (++n < 30) requestAnimationFrame(tick); };
      requestAnimationFrame(tick);
      setInterval(apply, 250);
    })();
  `;
}

export const OVERFLOW_PROBE = `(() => {
  var d = document.documentElement;
  return {
    vOverflow: +(d.scrollHeight / d.clientHeight).toFixed(3),
    hOverflow: +(d.scrollWidth / d.clientWidth).toFixed(3),
  };
})()`;
