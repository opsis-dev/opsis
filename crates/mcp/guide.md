# Working on an Opsis project

Opsis turns recorded websites and terminals into videos. A project is a folder with a `project.opsis.json` file: a graph of nodes (browser actions, terminal actions, cameras, elements, sound, exports) joined by edges. You work on the project open in Opsis through these tools.

## Rules

- **Change the project only through Lua, with `run_lua`.** Never write `project.opsis.json` yourself: Opsis owns that file, and a change it didn't make skips the checks, skips undo and makes Opsis ask what to do with it.
- **Every other file in the project folder is yours to read and write directly**: scripts in `scripts/`, assets, notes. Exports land in `.opsis/out`; read them as files.
- **Each `run_lua` call is one undo step.** A script that fails or is stopped undoes its own changes to the project. `undo` and `redo` step through those changes.
- **Start runs and return; then wait with `wait_run`.** A run outlives the script that started it, and only one script runs at a time, so a script that waits for a long export blocks every other script. `return opsis.run.start().id` gives the run's id.
- **Look up one node type at a time** with `opsis.catalog[type]`; the whole catalogue is large. Scripts have a 64 MiB memory limit and no time limit.
- **Scripts reach only the project folder and never the network.** `os.execute`, `io.popen`, `os.getenv` and `package` don't exist; `require "name"` reads `scripts/name.lua`.
- **A script's answer is its returned value as JSON, its printed lines and its failure.** Return what you want to see.

## The `opsis` module

Values are read live and set directly. Each assignment is one edit, checked as the editor checks it.

### Nodes

- `opsis.nodes.list()`: every node, as handles.
- `opsis.nodes.get(id)`, or `opsis.nodes[id]`: one node's handle, or nil.
- `opsis.nodes.find(filter)`: nodes by type or label (a string), by `{ type = …, label = … }`, or by a function of the node.
- `opsis.nodes.add{ type = …, id = …, label = …, position = { x = …, y = … }, params = { … } }`: adds a node and returns its handle.
- `opsis.nodes.remove(list)`: removes a node or a list of nodes.
- `opsis.nodes.move{ [node] = { x = …, y = … } }`: moves nodes.
- `opsis.nodes.group(list, title)` and `opsis.nodes.ungroup(group)`.
- `opsis.nodes.paste{ nodes = …, edges = …, offset = { x = …, y = … } }`: pastes copied nodes and returns their handles.
- A handle has `id`, `type`, `label`, `position`, `size`, `disabled`, `params` (its settings, each one set directly: `node.params.url = "https://…"`), `inputs`, `outputs`, `exists`, `result` and `remove()`. `label`, `position`, `size`, `disabled` and `params` can be set. A node's label is also its camera, chapter and caption label.

### Edges and variables

- `opsis.edges.list()`, `opsis.edges.connect(source, source_port, target, target_port)` (returns the edge's id) and `opsis.edges.disconnect(edge)`. Ports have types; `opsis.catalog[type]` lists a type's ports.
- `opsis.variables.all()`, `opsis.variables.get(name)`, `opsis.variables.set(name, value)` and `opsis.variables.rename(from, to)`.

### The project and the catalogue

- `opsis.project.name` (can be set), `opsis.project.settings` (each one set directly), `opsis.project.workflow` (the whole graph), `opsis.project.replace(workflow)`, `opsis.project.save()` and `opsis.project.problems()`.
- `opsis.catalog[type]`: one node type's ports, settings and description; `opsis.catalog()` is the whole catalogue.
- `opsis.validate(workflow)`: checks a graph without changing the project.

### Fonts

- `opsis.fonts.list()`, `opsis.fonts.use(id)`, `opsis.fonts.add(font)` and `opsis.fonts.role(role, family)`.

### Compile, timeline and mixer

- `opsis.compile()` or `opsis.compile.wait()`: compiles the current project and returns the result; `opsis.compile.issues()` and `opsis.compile.outdated()`.
- `opsis.timeline.flat()`, `opsis.timeline.spec()`, `opsis.timeline.edit(edit)`, `opsis.timeline.markers(edit)` and `opsis.timeline.keyframes(edit)`.
- `opsis.mixer.state()`, `opsis.mixer.plan()` and `opsis.mixer.edit(edit)`.

### Runs and exports

- `opsis.run.start{ nodes = { … } }` runs those nodes and what they need; `opsis.run.start{ from = node }` runs again from a node; `opsis.run.start()` runs everything that needs it. It returns a run with `id`, `run:events()` (each run event until `run:done`), `run:wait()`, `run:result(node)`, `run:missed()` and `run:stop()`.
- `opsis.run.stop()`, `opsis.run.active()`, `opsis.run.state()`, `opsis.run.log()`, `opsis.run.results()`, `opsis.run.result(node)` (with `.file`, the first file it made), `opsis.run.clear_results()` and `opsis.run.clear_log()`.
- An export is a run of an output node.

### Playback and selection

- `opsis.playback.seek(frame)`, `opsis.playback.play()`, `opsis.playback.pause()`, `opsis.playback.rate(rate)` and `opsis.playback.playhead()`.
- `opsis.selection.get()` and `opsis.selection.set(list)`.

### Values

- `opsis.list()` is an empty list (an empty table is sent as a map); `opsis.null` is JSON null; `opsis.version` is the Opsis version; `opsis.sleep(seconds)` waits.

## Tools

- `run_lua(source)`: runs a script on the open project and answers when it ends.
- `undo`, `redo`: step through the project's changes.
- `wait_run(id, seconds)`: waits up to that many seconds for a run to end and answers how it ended.
