--!strict

-- Runtime primitives used by trace-driven reconstruction/deobfuscation harnesses.
assert(type(loadstring) == "function", "loadstring is required by generated analysis harnesses")

local fn, err = loadstring("return 40 + 2", "=lusa-tooling-test")
assert(fn ~= nil, tostring(err))
assert(fn() == 42)

local taskLib = require("@lune/task")
assert(type(taskLib) == "table")
assert(type(taskLib.wait) == "function")
assert(type(taskLib.spawn) == "function")

local env = getfenv(fn)
assert(type(env) == "table")
setfenv(fn, env)

print("LUSA_TOOLING_RUNTIME_PASS")
