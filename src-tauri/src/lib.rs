use std::sync::Mutex;

use libmacrocenter::input::InputSimulator;
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};
use tauri::Manager;

struct SimulatorState {
    simulator: Mutex<InputSimulator>,
}

#[tauri::command]
fn simulate_type_text(state: tauri::State<SimulatorState>, text: String) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    sim.type_text(&text).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_key_action(
    state: tauri::State<SimulatorState>,
    key: String,
    mode: String,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    let action_mode: ActionMode = mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| e.to_string())?;
    sim.key_action(&key, action_mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_click(
    state: tauri::State<SimulatorState>,
    button: String,
    mode: String,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    let btn: MouseButton = button
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| e.to_string())?;
    let action_mode: ActionMode = mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| e.to_string())?;
    sim.mouse_click(btn, action_mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_move(
    state: tauri::State<SimulatorState>,
    x: i32,
    y: i32,
    mode: String,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    let coord_mode: CoordinateMode = mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| e.to_string())?;
    sim.mouse_move(x, y, coord_mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_scroll(
    state: tauri::State<SimulatorState>,
    axis: String,
    amount: i32,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    let scroll_axis: ScrollAxis = axis
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| e.to_string())?;
    sim.scroll(scroll_axis, amount).map_err(|e| e.to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                window.show().unwrap();
                window.set_focus().unwrap();
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let simulator = InputSimulator::new().map_err(|e| e.to_string())?;
            app.manage(SimulatorState {
                simulator: Mutex::new(simulator),
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
