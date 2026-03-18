// build first: cd libmacrocenter-node && bun run build
// then run: node examples/basic.ts

import { InputSimulator, HotkeyManager } from "../libmacrocenter-node/index.js";

console.log("=== libmacrocenter Features Showcase ===");

// 1. Input Simulation
console.log("\n[1] Simulator capabilities:");
const sim = new InputSimulator();
// Type some text
// console.log("- Typing text:");
// sim.typeText("Hello from libmacrocenter!\nThis is typing simulation.\n");

// console.log("- Key actions (press, release, click):");
// sim.keyAction("shift", "press");
// sim.keyAction("a", "click"); // Types an uppercase A
// sim.keyAction("shift", "release");

// console.log("- Mouse movements:");
// sim.mouseMove(100, 100, "absolute"); // Moves to top left
// sim.mouseMove(50, 50, "relative");   // Moves 50px right and down

// console.log("- Mouse clicks:");
// sim.mouseClick("right", "click");

// console.log("- Scrolling:");
// sim.scroll("vertical", -5); // Scrolls down
// sim.scroll("horizontal", 2); // Scrolls right

//  2. Hotkey Management & Callbacks
console.log("\n[2] Hotkey Listening:");

const mgr = new HotkeyManager();

const shiftM = mgr.register("9");
// const f5 = mgr.register("Ctrl+Alt+Shift+F5");
const f5 = mgr.register("F5");

console.log(
  `Registered hotkeys: \n- Ctrl+Alt+Shift+M (id: ${shiftM})\n- Ctrl+Alt+Shift+F5 (id: ${f5})`,
);
console.log("\nListening for hotkeys... Press Ctrl+C to exit.");

// Using the background-thread callback system
mgr.listen((err, event) => {
  if (err) {
    console.error("Hotkey error:", err);
    return;
  }

  let name = "Unknown";
  if (event.id === shiftM) name = "Ctrl+Alt+Shift+M";
  else if (event.id === f5) name = "F6";

  console.log(`Hotkey event triggered: ${name} -> ${event.state}`);

  if (event.state === "pressed") {
    setTimeout(() => {
      sim.typeText(`You pressed ${name}!\n`);
      sim.mouseMove(10, 10, "relative");
    }, 100);
  }
});

// in some conditions, we keep the script running indefinitely:
// setInterval(() => {}, 1000 * 60 * 60);
