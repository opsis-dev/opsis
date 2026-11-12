import { publicFile } from "@opsis/runtime/assets.ts";
import { Preloaded, SLATE, cell } from "#oracle/parts.tsx";

const PHOTO = publicFile("placeholders/launch-earth.jpg");
const LOGO = publicFile("placeholders/logo-mark.svg");
const WIDE = publicFile("placeholders/screen-wide.svg");

export function Case() {
  return (
    <Preloaded urls={[PHOTO, LOGO, WIDE]}>
      <div style={cell(0, { background: SLATE })}>
        <img src={PHOTO} alt="" style={{ width: "100%" }} />
      </div>
      <div style={cell(1, { background: SLATE, display: "grid", placeItems: "center" })}>
        <img src={LOGO} alt="" style={{ width: 80, height: 80 }} />
      </div>
      <div style={cell(2, { background: SLATE })}>
        <img src={WIDE} alt="" style={{ width: "100%", height: "100%", objectFit: "contain" }} />
      </div>
      <div style={cell(3, { background: SLATE, overflow: "hidden", borderRadius: 20 })}>
        <img
          src={PHOTO}
          alt=""
          style={{ width: "100%", height: "100%", objectFit: "cover", maxWidth: "none" }}
        />
      </div>
      <div style={cell(4, { background: SLATE })}>
        <img
          src={PHOTO}
          alt=""
          style={{ width: 40, height: 40, objectFit: "cover", imageRendering: "pixelated" }}
        />
        <img src={PHOTO} alt="" style={{ width: 300, maxWidth: "100%" }} />
      </div>
      <div style={cell(5, { background: SLATE, display: "grid", placeItems: "center" })}>
        <img src={LOGO} alt="" style={{ width: 120, height: 120, opacity: 0.5, rotate: "15deg" }} />
      </div>
      <div style={cell(6, { background: SLATE })}>
        <img
          src={PHOTO}
          alt=""
          style={{
            width: "100%",
            height: "100%",
            objectFit: "cover",
            maxWidth: "none",
            filter: "blur(3px) brightness(0.8)",
            scale: 1.2,
          }}
        />
      </div>
      <div style={cell(7, { background: SLATE })}>
        <img src={PHOTO} alt="" style={{ width: 120, height: 600, maxWidth: "none" }} />
      </div>
    </Preloaded>
  );
}
