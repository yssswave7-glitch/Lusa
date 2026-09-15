-- The default profile exposes bounded Potassium-compatible identity,
-- environment, and closure-query behavior.

local name, version = identifyexecutor()
assert(name == "potassium", "unexpected default executor name")
assert(version == "v2.4.8", "unexpected default executor version")
assert(getgenv() == getgenv(), "executor environment must be stable")
assert(getrenv() == getrenv(), "offline Roblox environment must be stable")
assert(getrenv() ~= getgenv(), "executor and Roblox environments must be distinct")
assert(getrenv().print == print, "offline Roblox environment must resolve runtime globals")
assert(iscclosure(identifyexecutor), "native identity callback must be a C closure")
assert(islclosure(function() end), "Luau closure classification failed")
assert(isexecutorclosure(function() end), "executor closure classification failed")
assert(not isexecutorclosure(print), "runtime-native closures are not executor closures")
assert(checkcaller(), "offline caller should be executor-owned")
local wrapped = newcclosure(function(a, b) return a + b, "ok" end)
local sum, marker = wrapped(2, 3)
assert(sum == 5 and marker == "ok", "newcclosure forwarding failed")
assert(iscclosure(wrapped), "newcclosure must return a native callback")

-- Unsafe/live-engine surfaces remain outside the offline compatibility layer.
assert(isfunctionhooked == nil, "hook-state spoofing must stay disabled")
assert(hookfunction == nil, "live hook emulation must stay disabled")
assert(request == nil, "network request emulation must stay disabled")
assert(readfile == nil and writefile == nil, "executor filesystem must stay disabled")

print("LUSA_EXECUTOR_BOUNDARY_PASS")
