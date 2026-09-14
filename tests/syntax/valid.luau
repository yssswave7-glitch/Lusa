--!strict
local value: number = 42
local function twice(n: number): number
    return n * 2
end
assert(twice(value) == 84)
print("LUSA_VALID_SYNTAX_PASS")
