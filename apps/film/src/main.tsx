import { mountFilm } from "@opsis/runtime/page.tsx";
import { FILM } from "@opsis/video/film";

const container = document.getElementById("film");
if (!container) throw new Error("The film page has no place for the film.");
mountFilm(container, FILM);
