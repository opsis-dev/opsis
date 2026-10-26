import { createOpsis } from "./opsis";
import { forwardConsole } from "./page-log";

forwardConsole();
window.opsis = createOpsis();
