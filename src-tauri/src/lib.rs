use std::sync::Mutex;

mod macro_export;
mod shell_command;

use libmacrocenter::input::InputSimulator;
use libmacrocenter::recording::{
    MousePositionEvent, RdevRecorder, RecordedMacro, RecorderBackend, RecordingMouseMode,
};
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};
use macro_export::{
    PORTABLE_MACRO_FILE, StartupMacroSource, StartupMacroState, export_portable_macro_bundle,
    export_standalone_macro, get_startup_macro, load_startup_macro,
};
use serde::Serialize;
use shell_command::execute_shell_command;
use tauri::AppHandle;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{
    MouseButton as TrayMouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

struct SimulatorState {
    simulator: Mutex<InputSimulator>,
}

struct RecorderState {
    recorder: RdevRecorder,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MacroRecordingStartedPayload {
    mode: RecordingMouseMode,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MacroRecordingStoppedPayload {
    macro_data: Option<RecordedMacro>,
    error: Option<String>,
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
    stop_recording_for_editor(&state)
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

fn stop_recording_for_editor(state: &RecorderState) -> Result<RecordedMacro, String> {
    let mut recorded = state.recorder.stop_recording().map_err(|e| e.to_string())?;
    recorded.trim_trailing_mouse_click();
    recorded.trim_trailing_recording_shortcut();
    Ok(recorded)
}

fn toggle_global_recording(app: &AppHandle, mode: RecordingMouseMode) {
    let state = app.state::<RecorderState>();
    if state.recorder.is_recording() {
        let payload = match stop_recording_for_editor(&state) {
            Ok(recorded) => MacroRecordingStoppedPayload {
                macro_data: Some(recorded),
                error: None,
            },
            Err(error) => MacroRecordingStoppedPayload {
                macro_data: None,
                error: Some(error),
            },
        };
        let _ = app.emit("macro_recording_stopped", payload);
        return;
    }

    let payload = match state.recorder.start_recording(mode) {
        Ok(()) => {
            let _ = app.emit(
                "macro_recording_started",
                MacroRecordingStartedPayload { mode },
            );
            return;
        }
        Err(error) => MacroRecordingStoppedPayload {
            macro_data: None,
            error: Some(error.to_string()),
        },
    };
    let _ = app.emit("macro_recording_stopped", payload);
}

fn show_editor(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_editor(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn run() {
    let startup_macro = load_startup_macro();
    let startup_macro_json = startup_macro
        .as_ref()
        .map(|startup_macro| startup_macro.macro_json.clone());
    let startup_macro_active = startup_macro_json.is_some();

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_editor(app);
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(move |app| {
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
            app.manage(StartupMacroState {
                macro_json: startup_macro_json,
            });

            app.global_shortcut()
                .on_shortcut("Ctrl+Shift+F1", |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_global_recording(app, RecordingMouseMode::MovesBeforeClicks);
                    }
                })
                .map_err(|e| e.to_string())?;
            app.global_shortcut()
                .on_shortcut("Ctrl+Shift+F2", |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_global_recording(app, RecordingMouseMode::AllMoves);
                    }
                })
                .map_err(|e| e.to_string())?;

            use tauri_plugin_notification::NotificationExt;
            if startup_macro_active {
                hide_editor(app.handle());

                let source = startup_macro
                    .as_ref()
                    .map(|macro_data| match &macro_data.source {
                        StartupMacroSource::Embedded => "Standalone macro".to_string(),
                        StartupMacroSource::AdjacentFile(path) => path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or(PORTABLE_MACRO_FILE)
                            .to_string(),
                    })
                    .unwrap_or_else(|| "macro".to_string());
                app.notification()
                    .builder()
                    .title("Macro Center")
                    .body(format!(
                        "{source} is running. Use the system tray to open the editor or quit."
                    ))
                    .show()
                    .unwrap();
            } else {
                show_editor(app.handle());
            }

            let open_i = MenuItem::with_id(app, "open", "Open Editor", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide Editor", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit Macro Center", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &hide_i, &quit_i])?;
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: TrayMouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        show_editor(app);
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        show_editor(app);
                    }
                    "hide" => {
                        hide_editor(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {
                        println!("Unknown menu item clicked: {:?}", event.id);
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
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
            is_mouse_position_monitoring,
            execute_shell_command,
            get_startup_macro,
            export_standalone_macro,
            export_portable_macro_bundle
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
