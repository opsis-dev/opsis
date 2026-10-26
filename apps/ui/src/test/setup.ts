import type { EditorCatalog } from "@/generated/bindings";
import { applyCatalog } from "@/store/catalog";
import { editorData } from "./editor-data";

applyCatalog({ catalog: editorData("catalog") as EditorCatalog, menu: [], templates: [] });
