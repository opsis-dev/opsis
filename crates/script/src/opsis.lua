local natives, plain = ...

local host = {}
for name, value in pairs(natives) do
  host[name] = type(value) == "function" and plain(value) or value
end

local opsis = {}
local ids = setmetatable({}, { __mode = "k" })

local function fail(message)
  return host.fail(message)
end

local function id_of(value, what)
  if type(value) == "string" then
    return value
  end
  local id = type(value) == "table" and ids[value]
  if id then
    return id
  end
  if type(value) == "table" and type(value.id) == "string" then
    return value.id
  end
  return fail("Expected " .. what .. " or its id, got " .. type(value) .. ".")
end

local function list_of(value)
  if value == nil or next(value) == nil then
    return host.list()
  end
  return value
end

local function current(id)
  local node = host.node(id)
  if node == nil then
    return fail("This node is gone.")
  end
  return node
end

local function update(id, patch)
  current(id)
  host.edit({ op = "updateNode", id = id, patch = patch })
end

local function params_proxy(id)
  local proxy = setmetatable({}, {
    __index = function(_, key)
      return current(id).params[key]
    end,
    __newindex = function(_, key, value)
      if value == nil then
        return fail("A setting can't be nil; set it to the value you want.")
      end
      update(id, { params = { [key] = value } })
    end,
    __pairs = function()
      return next, current(id).params, nil
    end,
    __tostring = function()
      return "settings of node " .. id
    end,
  })
  return proxy
end

local node_meta = {}

local node_writes = { label = true, position = true, disabled = true, size = true }

local function node_proxy(id)
  local proxy = setmetatable({}, node_meta)
  ids[proxy] = id
  return proxy
end

node_meta.__index = function(proxy, key)
  local id = ids[proxy]
  if key == "id" then
    current(id)
    return id
  elseif key == "params" then
    current(id)
    return params_proxy(id)
  elseif key == "exists" then
    return host.node(id) ~= nil
  elseif key == "remove" then
    return function()
      current(id)
      host.edit({ op = "removeNodes", ids = { id } })
    end
  elseif key == "inputs" or key == "outputs" then
    current(id)
    local found = {}
    for _, edge in ipairs(host.workflow().edges) do
      if (key == "inputs" and edge.target == id) or (key == "outputs" and edge.source == id) then
        found[#found + 1] = edge
      end
    end
    return found
  elseif key == "result" then
    current(id)
    return opsis.run.result(id)
  end
  return current(id)[key]
end

node_meta.__newindex = function(proxy, key, value)
  local id = ids[proxy]
  if key == "params" then
    if type(value) ~= "table" then
      return fail("A node's params must be a table.")
    end
    update(id, { params = value })
  elseif key == "label" then
    update(id, { label = value or "" })
  elseif node_writes[key] then
    update(id, { [key] = value })
  else
    return fail("A node has no field " .. tostring(key) .. " a script can set.")
  end
end

node_meta.__eq = function(a, b)
  return ids[a] ~= nil and ids[a] == ids[b]
end

node_meta.__tostring = function(proxy)
  return "node " .. ids[proxy]
end

local function node_list()
  local list = host.list()
  for _, node in ipairs(host.workflow().nodes) do
    list[#list + 1] = node_proxy(node.id)
  end
  return list
end

local nodes = {}

function nodes.get(id)
  if host.node(id) == nil then
    return nil
  end
  return node_proxy(id)
end

function nodes.list()
  return node_list()
end

function nodes.find(filter)
  local found = host.list()
  for _, node in ipairs(host.workflow().nodes) do
    local keep
    if type(filter) == "function" then
      keep = filter(node)
    elseif type(filter) == "table" then
      keep = (filter.type == nil or filter.type == node.type) and (filter.label == nil or filter.label == node.label)
    else
      keep = node.type == filter or node.label == filter
    end
    if keep then
      found[#found + 1] = node_proxy(node.id)
    end
  end
  return found
end

function nodes.add(spec)
  if type(spec) ~= "table" or type(spec.type) ~= "string" then
    return fail("opsis.nodes.add needs a table with a type.")
  end
  local edited = host.edit({
    op = "addNode",
    type = spec.type,
    id = spec.id,
    label = spec.label,
    position = spec.position,
    params = spec.params,
  })
  return node_proxy(edited.ids[1])
end

function nodes.remove(list)
  local removed = {}
  if type(list) ~= "table" or ids[list] then
    list = { list }
  end
  for i, value in ipairs(list) do
    removed[i] = id_of(value, "a node")
  end
  host.edit({ op = "removeNodes", ids = removed })
end

function nodes.move(positions)
  local moved = {}
  for key, position in pairs(positions) do
    moved[id_of(key, "a node")] = position
  end
  host.edit({ op = "moveNodes", positions = moved })
end

function nodes.group(list, title)
  local grouped, sizes = {}, {}
  for i, value in ipairs(list) do
    grouped[i] = id_of(value, "a node")
  end
  local edited = host.edit({ op = "group", ids = grouped, title = title or "Group", sizes = sizes })
  if edited.ids[1] == nil then
    return fail("A group needs at least two nodes that aren't in a group already.")
  end
  return node_proxy(edited.ids[1])
end

function nodes.ungroup(group)
  host.edit({ op = "ungroup", id = id_of(group, "a group") })
end

function nodes.paste(spec)
  local edited = host.edit({
    op = "paste",
    nodes = list_of(spec.nodes),
    edges = list_of(spec.edges),
    offset = spec.offset or { x = 0, y = 0 },
  })
  local pasted = host.list()
  for i, id in ipairs(edited.ids) do
    pasted[i] = node_proxy(id)
  end
  return pasted
end

opsis.nodes = setmetatable(nodes, {
  __index = function(_, id)
    if type(id) == "string" and host.node(id) ~= nil then
      return node_proxy(id)
    end
    return nil
  end,
  __newindex = function()
    return fail("Add nodes with opsis.nodes.add.")
  end,
  __len = function()
    return #host.workflow().nodes
  end,
  __pairs = function()
    local list, i = node_list(), 0
    return function()
      i = i + 1
      local node = list[i]
      if node then
        return ids[node], node
      end
    end
  end,
})

opsis.edges = {}

function opsis.edges.list()
  return host.workflow().edges
end

function opsis.edges.connect(source, source_port, target, target_port)
  local edited = host.edit({
    op = "addEdge",
    ends = {
      source = id_of(source, "a node"),
      sourceHandle = source_port,
      target = id_of(target, "a node"),
      targetHandle = target_port,
    },
  })
  return edited.ids[1]
end

function opsis.edges.disconnect(edge)
  host.edit({ op = "removeEdges", ids = { id_of(edge, "a connection") } })
end

opsis.variables = {}

function opsis.variables.all()
  return host.workflow().variables
end

function opsis.variables.get(name)
  return host.workflow().variables[name]
end

function opsis.variables.set(name, value)
  local all = host.workflow().variables
  all[name] = value
  host.edit({ op = "setVariables", variables = all })
end

function opsis.variables.rename(from, to)
  host.edit({ op = "renameVariable", from = from, to = to })
end

local settings = setmetatable({}, {
  __index = function(_, key)
    return host.project().settings[key]
  end,
  __newindex = function(_, key, value)
    host.edit({ op = "setSettings", patch = { [key] = value } })
  end,
  __pairs = function()
    return next, host.project().settings, nil
  end,
})

opsis.project = setmetatable({}, {
  __index = function(_, key)
    if key == "settings" then
      return settings
    elseif key == "save" then
      return host.save
    elseif key == "problems" then
      return host.problems
    elseif key == "workflow" then
      return host.workflow()
    elseif key == "replace" then
      return function(workflow)
        host.edit({ op = "replace", workflow = workflow })
      end
    end
    return host.project()[key]
  end,
  __newindex = function(_, key, value)
    if key == "name" then
      host.edit({ op = "rename", name = value })
    else
      return fail("The project has no field " .. tostring(key) .. " a script can set.")
    end
  end,
})

opsis.catalog = setmetatable({}, {
  __index = function(_, key)
    for _, entry in ipairs(host.catalog()) do
      if entry.type == key then
        return entry
      end
    end
    return nil
  end,
  __call = function()
    return host.catalog()
  end,
})

opsis.fonts = {}

function opsis.fonts.list()
  return host.fonts()
end

function opsis.fonts.use(id)
  return host.use_font(id)
end

function opsis.fonts.add(font)
  host.edit({ op = "addProjectFont", font = font })
end

function opsis.fonts.role(role, family)
  host.edit({ op = "setFontRole", role = role, family = family })
end

opsis.compile = setmetatable({}, {
  __call = function()
    return host.compile()
  end,
})

function opsis.compile.wait()
  return host.compile()
end

function opsis.compile.issues()
  return host.compile().issues
end

function opsis.compile.outdated()
  return host.compile().outdated
end

opsis.timeline = {}

function opsis.timeline.flat()
  return host.compile().flat
end

function opsis.timeline.spec()
  return host.compile().levelled
end

function opsis.timeline.edit(edit)
  host.compile()
  return host.timeline_edit(edit)
end

function opsis.timeline.markers(edit)
  host.edit({ op = "markers", edit = edit })
end

function opsis.timeline.keyframes(edit)
  host.edit({ op = "keyframes", edit = edit })
end

opsis.mixer = {}

function opsis.mixer.state()
  return host.compile().mix
end

function opsis.mixer.plan()
  return host.compile().plan
end

function opsis.mixer.edit(edit)
  host.edit({ op = "mix", edit = edit })
end

local run_meta = {}

local function result_of(id)
  local result = host.results()[id]
  if result ~= nil and result.artifacts ~= nil and result.artifacts[1] ~= nil then
    result.file = result.artifacts[1].path
  end
  return result
end

run_meta.__index = {
  events = function(run)
    return function()
      return host.run_next(run.handle)
    end
  end,
  wait = function(run)
    while host.run_next(run.handle) ~= nil do
    end
    return host.run_state()
  end,
  result = function(_, id)
    return result_of(id_of(id, "a node"))
  end,
  missed = function(run)
    return host.run_missed(run.handle)
  end,
  stop = function()
    host.run_stop()
  end,
}

opsis.browser = {}

function opsis.browser.open(up_to)
  return host.browser_open(id_of(up_to, "a browser action"))
end

function opsis.browser.state()
  local state = host.browser_state()
  if state == host.null then
    return nil
  end
  return state
end

function opsis.browser.check(up_to)
  return host.browser_check(id_of(up_to, "a browser action"))
end

function opsis.browser.close()
  host.browser_close()
end

opsis.run = {}

function opsis.run.start(spec)
  spec = spec or {}
  local targets = {}
  if spec.nodes ~= nil then
    local chosen = host.list()
    for i, value in ipairs(spec.nodes) do
      chosen[i] = id_of(value, "a node")
    end
    targets.targetNodeIds = chosen
  end
  if spec.from ~= nil then
    targets.fromNodeId = id_of(spec.from, "a node")
  end
  local handle = host.run_start(targets)
  return setmetatable({ id = host.run_id(handle), handle = handle }, run_meta)
end

function opsis.run.stop()
  host.run_stop()
end

function opsis.run.active()
  return host.run_active()
end

function opsis.run.state()
  return host.run_state()
end

function opsis.run.log()
  return host.run_state().logs
end

function opsis.run.results()
  return host.results()
end

function opsis.run.result(id)
  return result_of(id_of(id, "a node"))
end

function opsis.run.clear_results()
  host.clear_results()
end

function opsis.run.clear_log()
  host.clear_log()
end

opsis.playback = {}

function opsis.playback.seek(frame)
  host.playback({ call = "seek", frame = frame })
end

function opsis.playback.play()
  host.playback({ call = "play" })
end

function opsis.playback.pause()
  host.playback({ call = "pause" })
end

function opsis.playback.rate(rate)
  host.playback({ call = "rate", rate = rate })
end

function opsis.playback.playhead()
  return host.shared().playhead
end

opsis.selection = {}

function opsis.selection.get()
  return host.shared().selection.selectedIds
end

function opsis.selection.set(list)
  local chosen = host.list()
  for i, value in ipairs(list) do
    chosen[i] = id_of(value, "a node")
  end
  host.select(chosen)
end

function opsis.validate(workflow)
  return host.validate(workflow)
end

function opsis.sleep(seconds)
  host.sleep(seconds)
end

function opsis.list()
  return host.list()
end

opsis.null = host.null
opsis.version = host.version()

return opsis
