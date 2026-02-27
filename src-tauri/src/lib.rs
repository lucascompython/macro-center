use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

struct EnigoState {
    enigo: Mutex<Enigo>,
}

#[tauri::command]
fn simulate_type_text(state: tauri::State<EnigoState>, text: String) -> Result<(), String> {
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;
    enigo.text(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_key_action(
    state: tauri::State<EnigoState>,
    key: String,
    mode: String,
) -> Result<(), String> {
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;

    // parse the key string to Enigo key
    // this is a naive implementation, a more robust one would map strings to Key enum variants
    // for MVP we might need to rely on Key::Unicode for chars or specific mapping

    // attempt to map common keys, fallback to unicode
    let key_enum = match key.to_lowercase().as_str() {
        "enter" | "return" => Key::Return,
        "tab" => Key::Tab,
        "space" => Key::Space,
        "backspace" => Key::Backspace,
        "escape" | "esc" => Key::Escape,
        "control" | "ctrl" => Key::Control,
        "shift" => Key::Shift,
        "alt" => Key::Alt,
        "meta" | "super" | "win" | "cmd" => Key::Meta,
        "arrowup" | "up" => Key::UpArrow,
        "arrowdown" | "down" => Key::DownArrow,
        "arrowleft" | "left" => Key::LeftArrow,
        "arrowright" | "right" => Key::RightArrow,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "home" => Key::Home,
        "end" => Key::End,
        "delete" | "del" => Key::Delete,
        "capslock" => Key::CapsLock,
        "f1" => Key::F1, "f2" => Key::F2, "f3" => Key::F3, "f4" => Key::F4,
        "f5" => Key::F5, "f6" => Key::F6, "f7" => Key::F7, "f8" => Key::F8,
        "f9" => Key::F9, "f10" => Key::F10, "f11" => Key::F11, "f12" => Key::F12,
        s if s.len() == 1 => Key::Unicode(s.chars().next().unwrap()),
        _ => return Err(format!("Unknown key: {}", key)),
    };

    let direction = match mode.as_str() {
        "click" => Direction::Click,
        "press" => Direction::Press,
        "release" => Direction::Release,
        _ => Direction::Click,
    };

    enigo.key(key_enum, direction).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_click(
    state: tauri::State<EnigoState>,
    button: String,
    mode: String,
) -> Result<(), String> {
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;

    let button_enum = match button.to_lowercase().as_str() {
        "left" => Button::Left,
        "right" => Button::Right,
        "middle" => Button::Middle,
        _ => Button::Left,
    };

    let direction = match mode.as_str() {
        "click" => Direction::Click,
        "press" => Direction::Press,
        "release" => Direction::Release,
        _ => Direction::Click,
    };

    enigo
        .button(button_enum, direction)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_move(
    state: tauri::State<EnigoState>,
    x: i32,
    y: i32,
    mode: String,
) -> Result<(), String> {
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;

    match mode.as_str() {
        "absolute" => enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| e.to_string()),
        "relative" => enigo
            .move_mouse(x, y, Coordinate::Rel)
            .map_err(|e| e.to_string()),
        _ => enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(|e| e.to_string()),
    }
}

#[tauri::command]
fn simulate_scroll(
    state: tauri::State<EnigoState>,
    axis: String,
    amount: i32,
) -> Result<(), String> {
    let mut enigo = state.enigo.lock().map_err(|e| e.to_string())?;

    let direction = match axis.to_lowercase().as_str() {
        "horizontal" => Axis::Horizontal,
        _ => Axis::Vertical,
    };

    enigo.scroll(amount, direction).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            app.manage(EnigoState {
                enigo: Mutex::new(enigo),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            simulate_type_text,
            simulate_key_action,
            simulate_mouse_click,
            simulate_mouse_move,
            simulate_scroll
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
