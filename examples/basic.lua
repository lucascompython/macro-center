-- build first: cd libmacrocenter-lua && cargo build
-- Copy the .so/.dll to your lua path, then run: lua examples/basic.lua

local mc = require("macrocenter_lua")

print("=== libmacrocenter Features Showcase ===")

--  1. Input Simulation
print("\n[1] Simulator capabilities:")
local sim = mc.InputSimulator()

-- print("- Typing text:")
-- sim:type_text("Hello from libmacrocenter!\nThis is typing simulation.\n")

-- print("- Key actions (press, release, click):")
-- sim:key_action("shift", "press")
-- sim:key_action("a", "click") -- Types an uppercase A
-- sim:key_action("shift", "release")

-- print("- Mouse movements:")
-- sim:mouse_move(100, 100, "absolute") -- Moves to top left
-- sim:mouse_move(50, 50, "relative")   -- Moves 50px right and down

-- print("- Mouse clicks:")
-- sim:mouse_click("right", "click")

-- print("- Scrolling:")
-- sim:scroll("vertical", -5) -- Scrolls down
-- sim:scroll("horizontal", 2) -- Scrolls right


--  2. Hotkey Management & Callbacks
print("\n[2] Hotkey Listening:")

local mgr = mc.HotkeyManager()

local shiftM = mgr:register("Ctrl+Alt+Shift+M")
local f5 = mgr:register("Ctrl+Alt+Shift+F5")

print(string.format("Registered hotkeys: \n- Ctrl+Alt+Shift+M (id: %d)\n- Ctrl+Alt+Shift+F5 (id: %d)", shiftM, f5))
print("\nListening for hotkeys... Press Ctrl+C to exit.")

-- Using the callback listening system
mgr:listen(function(event)
    local name = "Unknown"
    if event.id == shiftM then
        name = "Ctrl+Alt+Shift+M"
    elseif event.id == f5 then
        name = "Ctrl+Alt+Shift+F5"
    end

    print(string.format("Hotkey event triggered: %s -> %s", name, event.state))

    if event.state == "pressed" then
        print(" -> Performing simulated response...")
        sim:type_text(string.format("You pressed %s!\n", name))
        sim:mouse_move(10, 10, "relative")
    end
end)
