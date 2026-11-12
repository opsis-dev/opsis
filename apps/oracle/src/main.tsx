import { mountFilm } from "@opsis/runtime/page.tsx";
import { boxesOf } from "#oracle/boxes.ts";
import { CASE_FILM } from "#oracle/case-film.tsx";

const container = document.getElementById("film");
if (!container) throw new Error("The oracle page has no place for its case.");
mountFilm(container, CASE_FILM);
window.oracleBoxes = () => boxesOf(container);
