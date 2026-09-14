-- Executor-specific APIs are intentionally not fabricated by Lusa.
-- They are not Roblox engine globals and spoofing them would make compatibility
-- results misleading.

assert(identifyexecutor == nil, "executor identity spoofing must stay disabled")
assert(isfunctionhooked == nil, "hook-state spoofing must stay disabled")
assert(islclosure == nil, "closure-provenance spoofing must stay disabled")
assert(getgenv == nil, "executor global-environment API must stay disabled")

print("LUSA_EXECUTOR_BOUNDARY_PASS")
