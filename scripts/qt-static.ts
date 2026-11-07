import { QT_VERSION, ensureStaticQt } from "./lib/static-qt";

console.log(`Static Qt ${QT_VERSION} without ICU: ${await ensureStaticQt()}`);
