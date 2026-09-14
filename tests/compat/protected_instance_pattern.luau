-- Reduced regression for the Instance-integrity pattern used by protected
-- Roblox scripts. A mismatch changes the guard key and fails much later.

local guard = 0

if type(game) ~= "userdata" then
	guard = bit32.bxor(guard, 4186987685)
end

if pcall(rawget, game, "___probe") then
	guard = bit32.bxor(guard, 123456789)
end

local metatable = getmetatable(game)
if metatable ~= nil and metatable ~= "The metatable is locked" then
	guard = bit32.bxor(guard, 849506172)
end

local ok, child = pcall(function()
	return game:FindFirstChild("__lusa_missing_canary")
end)
if ok and child ~= nil then
	guard = bit32.bxor(guard, 738495061)
end

assert(guard == 0, `Instance fidelity guard changed: {guard}`)
assert(debug.info(getmetatable, "s") == "[C]")
assert(game:GetService("Players") == game:GetService("Players"))

print("LUSA_PROTECTED_INSTANCE_PATTERN_PASS")

