--!strict
-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at https://mozilla.org/MPL/2.0/.
--
-- Lusa Roblox compatibility bootstrap.
-- Scope: Roblox/Luau language + offline engine-surface compatibility.
-- Deliberately out of scope: exploit-executor identity spoofing, fake hook
-- state, forged native/C closure provenance, or stack-frame concealment.

local roblox = require("@lune/roblox")
local taskLib = require("@lune/task")
local DateTimeLib = require("@lune/datetime")

local function createSignal()
	local listeners = {}
	local signal = {}

	function signal:Connect(callback)
		assert(type(callback) == "function", "Connect expects a function")
		local connection = { Connected = true }
		function connection:Disconnect()
			if not self.Connected then
				return
			end
			self.Connected = false
			listeners[connection] = nil
		end
		listeners[connection] = callback
		return connection
	end

	function signal:Once(callback)
		local connection
		connection = self:Connect(function(...)
			connection:Disconnect()
			callback(...)
		end)
		return connection
	end

	return table.freeze(signal)
end

local playerAdded = createSignal()
local playerRemoving = createSignal()

pcall(function()
	roblox.implementProperty("Players", "PlayerAdded", function(_players)
		return playerAdded
	end)
end)

pcall(function()
	roblox.implementProperty("Players", "PlayerRemoving", function(_players)
		return playerRemoving
	end)
end)

-- Roblox exposes Workspace:GetServerTimeNow() as a number in seconds. Outside
-- a live server there is no authoritative server clock, so Lusa uses wall-clock
-- time. The observable type/forward-progress contract is preserved, but the
-- value is not synchronized with a Roblox server.
pcall(function()
    roblox.implementMethod("Workspace", "GetServerTimeNow", function(_workspace)
        return DateTimeLib.now().unixTimestampMillis / 1000
    end)
end)

-- Common DataModel behavior useful to ordinary scripts.
pcall(function()
    roblox.implementMethod("DataModel", "IsLoaded", function(_game)
        return true
    end)
end)

local game = roblox.Instance.new("DataModel")
local workspace = game:GetService("Workspace")
local globals = getfenv(0)

globals.game = game
globals.workspace = workspace
globals.task = taskLib
globals.DateTime = DateTimeLib

-- Export Roblox datatypes / constructors as globals (Instance, Enum, Vector3,
-- CFrame, Color3, UDim2, etc.) when supplied by @lune/roblox.
for name, value in roblox do
    if type(name) == "string" and string.match(name, "^[A-Z]") then
        globals[name] = value
    end
end

-- Keep the canonical constructor explicit even if module exports move around.
globals.Instance = roblox.Instance
