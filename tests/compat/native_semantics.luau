-- Luau builtin behavior exercised by common compatibility / integrity checks.

local pcall0, type0, rawget0, getmetatable0 = pcall, type, rawget, getmetatable
local bitlib = bit32 or bit
local concat0, char0, byte0 = table.concat, string.char, string.byte

assert(type0(type0) == "function")
assert(type0(pcall0) == "function")
assert(type0(rawget0) == "function")
assert(type0(getmetatable0) == "function")
assert(type0(bitlib) == "table")

local canaryTable = setmetatable({}, { __index = function() return "canary" end })
assert(rawget0(canaryTable, "canary") == nil)

if debug and type(debug) == "table" and type(debug.info) == "function" then
    assert(debug.info(pcall0, "s") == "[C]")
    assert(debug.info(type0, "s") == "[C]")
    assert(debug.info(byte0, "s") == "[C]")
    assert(debug.info(char0, "s") == "[C]")
    assert(debug.info(concat0, "s") == "[C]")
    assert(debug.info(bitlib.bxor, "s") == "[C]")
end

assert(bitlib.bxor(12345, 54321) == 58376)
assert(byte0("A") == 65)
assert(char0(65) == "A")

if table and type(table.isfrozen) == "function" then
    assert(table.isfrozen(buffer))
    assert(table.isfrozen(string))
    assert(table.isfrozen(table))
    assert(table.isfrozen(bitlib))
end

print("LUSA_NATIVE_SEMANTICS_PASS")
