use libmacrocenter::hotkey::HotkeyManager;
use libmacrocenter::input::InputSimulator;
#[allow(unused_imports)]
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Input Simulation
    println!("=== Input Simulation ===");

    let mut _sim = InputSimulator::new()?;

    // Type some text
    // println!("Typing 'Hello from libmacrocenter!'...");
    // sim.type_text("Hello from libmacrocenter!")?;

    // Press a key
    // println!("Pressing Enter...");
    // sim.key_action("enter", ActionMode::Click)?;

    // Mouse actions
    // println!("Moving mouse to (500, 500)...");
    // sim.mouse_move(500, 500, CoordinateMode::Absolute)?;

    // println!("Clicking left mouse button...");
    // sim.mouse_click(MouseButton::Left, ActionMode::Click)?;

    // println!("Scrolling down...");
    // sim.scroll(ScrollAxis::Vertical, -3)?;

    // NOTE: The above calls are commented out to avoid actually
    // sending input when running the example. Uncomment them to
    // see them in action!

    // --- Hotkey Listening ---
    // println!("\n=== Hotkey Listening ===");

    // NOTE: global-hotkey requires an event loop on the current thread.
    // on Linux (X11), this example should work. On macOS/Windows,
    // you need a proper event loop running.
    let mut mgr = HotkeyManager::new()?;

    // register a hotkey
    let id = mgr.register("Ctrl+Shift+M")?;
    println!("Registered hotkey Ctrl+Shift+M with id: {id}");

    let f5_id = mgr.register("F5")?;
    println!("Registered hotkey F5 with id: {f5_id}");

    println!("\nListening for hotkeys... Press Ctrl+C to exit.");
    println!("Try pressing Ctrl+Shift+M or F5!\n");

    loop {
        if let Some(event) = mgr.poll_event() {
            let name = if event.id == id {
                "Ctrl+Shift+M"
            } else if event.id == f5_id {
                "F5"
            } else {
                "Unknown"
            };
            println!("Hotkey event: {name} -> {:?}", event.state);
        }

        thread::sleep(Duration::from_millis(10));
    }
}
