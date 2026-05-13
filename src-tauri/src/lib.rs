use std::sync::Mutex;

use libmacrocenter::input::InputSimulator;
use libmacrocenter::recording::{
    MousePositionEvent, RdevRecorder, RecordedMacro, RecorderBackend, RecordingMouseMode,
};
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};
use tauri::{Emitter, Manager};

struct SimulatorState {
    simulator: Mutex<InputSimulator>,
}

struct RecorderState {
    recorder: RdevRecorder,
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
    mode: ActionMode,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    sim.key_action(&key, mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_click(
    state: tauri::State<SimulatorState>,
    button: MouseButton,
    mode: ActionMode,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    sim.mouse_click(button, mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_mouse_move(
    state: tauri::State<SimulatorState>,
    x: i32,
    y: i32,
    mode: CoordinateMode,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    sim.mouse_move(x, y, mode).map_err(|e| e.to_string())
}

#[tauri::command]
fn simulate_scroll(
    state: tauri::State<SimulatorState>,
    axis: ScrollAxis,
    amount: i32,
) -> Result<(), String> {
    let mut sim = state.simulator.lock().map_err(|e| e.to_string())?;
    sim.scroll(axis, amount).map_err(|e| e.to_string())
}

#[tauri::command]
fn start_macro_recording(
    state: tauri::State<RecorderState>,
    mode: RecordingMouseMode,
) -> Result<(), String> {
    state
        .recorder
        .start_recording(mode)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_macro_recording(state: tauri::State<RecorderState>) -> Result<RecordedMacro, String> {
    let mut recorded = state.recorder.stop_recording().map_err(|e| e.to_string())?;
    recorded.trim_trailing_mouse_click();
    Ok(recorded)
}

#[tauri::command]
fn is_macro_recording(state: tauri::State<RecorderState>) -> bool {
    state.recorder.is_recording()
}

#[tauri::command]
fn start_mouse_position_monitor(
    app: tauri::AppHandle,
    state: tauri::State<RecorderState>,
) -> Result<(), String> {
    state
        .recorder
        .start_mouse_listening(move |event: MousePositionEvent| {
            let _ = app.emit("mouse_position", event);
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_mouse_position_monitor(state: tauri::State<RecorderState>) {
    state.recorder.stop_mouse_listening();
}

#[tauri::command]
fn is_mouse_position_monitoring(state: tauri::State<RecorderState>) -> bool {
    state.recorder.is_mouse_listening()
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
            #[cfg(target_os = "linux")]
            {
                let gtk_csd = std::env::var("GTK_CSD").unwrap_or_default();
                let qt_csd =
                    std::env::var("QT_WAYLAND_DISABLE_WINDOWDECORATION").unwrap_or_default();

                if (gtk_csd == "0" || qt_csd == "1")
                    && let Some(window) = app.get_webview_window("main")
                {
                    window.set_decorations(false).unwrap();
                }
            }

            let simulator = InputSimulator::new().map_err(|e| e.to_string())?;
            app.manage(SimulatorState {
                simulator: Mutex::new(simulator),
            });
            app.manage(RecorderState {
                recorder: RdevRecorder::new(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            simulate_type_text,
            simulate_key_action,
            simulate_mouse_click,
            simulate_mouse_move,
            simulate_scroll,
            start_macro_recording,
            stop_macro_recording,
            is_macro_recording,
            start_mouse_position_monitor,
            stop_mouse_position_monitor,
            is_mouse_position_monitoring
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
