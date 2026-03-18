"""
build first: cd libmacrocenter-python && maturin develop
then run:    python examples/basic.py
"""

import time

import macrocenter_python as mc

print("=== libmacrocenter Features Showcase ===")

#  1. Input Simulation
print("\n[1] Simulator capabilities:")
sim = mc.InputSimulator()

# print("- Typing text:")
# sim.type_text("Hello from libmacrocenter!\nThis is typing simulation.\n")

# print("- Key actions (press, release, click):")
# sim.key_action("shift", "press")
# sim.key_action("a", "click") # Types an uppercase A
# sim.key_action("shift", "release")

# print("- Mouse movements:")
# sim.mouse_move(100, 100, "absolute") # Moves to top left
# sim.mouse_move(50, 50, "relative")   # Moves 50px right and down

# print("- Mouse clicks:")
# sim.mouse_click("right", "click")

# print("- Scrolling:")
# sim.scroll("vertical", -5) # Scrolls down
# sim.scroll("horizontal", 2) # Scrolls right

# 2. Hotkey Management & Callbacks
print("\n[2] Hotkey Listening:")

mgr = mc.HotkeyManager()

shiftM = mgr.register("Ctrl+Alt+Shift+M")
f5 = mgr.register("F5")

print(
    f"Registered hotkeys: \n- Ctrl+Alt+Shift+M (id: {shiftM})\n- Ctrl+Alt+Shift+F5 (id: {f5})"
)
print("\nListening for hotkeys... Press Ctrl+C to exit.")


def on_hotkey(event):
    name = "Unknown"
    if event["id"] == shiftM:
        name = "Ctrl+Alt+Shift+M"
    elif event["id"] == f5:
        name = "F5"

    print(f"Hotkey event triggered: {name} -> {event['state']}")

    if event["state"] == "pressed":
        print(" -> Performing simulated response...")
        # TODO: atleast on linux (xorg) without this time.sleep the type_text wont get actually typed
        time.sleep(0.1)
        sim.type_text(f"You pressed {name}!")
        sim.mouse_move(10, 10, "relative")


try:
    mgr.listen(on_hotkey)
except KeyboardInterrupt:
    print("\nCleaning up...")
    mgr.unregister_all()
    print("Done!")
