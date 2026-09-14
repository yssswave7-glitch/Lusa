-- Engine-surface checks that Lusa intentionally supports.

assert(type(game) == "userdata", "game must be a DataModel-like Instance")
assert(type(workspace) == "userdata", "workspace must be a Workspace-like Instance")
assert(type(Instance) == "table", "Instance constructor must be exposed")
assert(type(task) == "table", "task must be exposed")

assert(game:IsLoaded() == true)
assert(workspace == game:GetService("Workspace"))

local t1 = workspace:GetServerTimeNow()
task.wait(0.05)
local t2 = workspace:GetServerTimeNow()
local delta = t2 - t1
assert(type(t1) == "number")
assert(type(t2) == "number")
assert(delta > 0 and delta < 5, `unexpected clock delta: {delta}`)

local slim = game:GetService("SlimAnimationReplicationService")
assert(slim ~= nil)
assert(slim.Name == "SlimAnimationReplicationService")

local ok, err = pcall(function()
    local inst = Instance.new("Folder")
    inst.Parent = game:GetService("CoreGui")
    local valid = inst:IsDescendantOf(game) and typeof(inst:GetDebugId()) == "string"
    inst:Destroy()
    assert(valid)
end)
assert(ok, tostring(err))

local env = getfenv()
local fn = function()
    return getfenv(1)
end
setfenv(fn, env)
assert(rawequal(getfenv(fn), env))

print("LUSA_ROBLOX_CORE_PASS")
