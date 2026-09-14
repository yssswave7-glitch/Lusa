--!strict

local service = game:GetService("ActivityHistoryEventService")
assert(service.ClassName == "ActivityHistoryEventService")
assert(service:IsA("Instance"))

local object = Instance.new("AccessoryDescription")
assert(object.ClassName == "AccessoryDescription")
assert(object:IsA("Instance"))

print("LUSA_API_REGISTRY_PASS")
