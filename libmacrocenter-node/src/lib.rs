#![deny(clippy::all)]

use napi::Result;
use napi_derive::napi;

use libmacrocenter::hotkey::{HotkeyManager, HotkeyState};
use libmacrocenter::input::InputSimulator;
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

/// A hotkey event returned from polling.
#[napi(object)]
pub struct JsHotkeyEvent {
  /// The hotkey ID.
  pub id: u32,
  /// The state: "pressed" or "released".
  pub state: String,
}

// TODO: instead of changing the js name just change the import
/// Input simulator for keyboard and mouse actions.
#[napi(js_name = "InputSimulator")]
pub struct JsInputSimulator {
  inner: InputSimulator,
}

#[napi]
impl JsInputSimulator {
  /// Create a new input simulator.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = InputSimulator::new().map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Type a string of text.
  #[napi]
  pub fn type_text(&mut self, text: String) -> Result<()> {
    self
      .inner
      .type_text(&text)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Perform a key action.
  ///
  /// `key`: Key name like "enter", "a", "F5", etc.
  /// `mode`: "click", "press", or "release".
  #[napi]
  pub fn key_action(&mut self, key: String, mode: String) -> Result<()> {
    let action_mode: ActionMode =
      mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| {
          napi::Error::from_reason(e.to_string())
        })?;
    self
      .inner
      .key_action(&key, action_mode)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Perform a mouse button action.
  ///
  /// `button`: "left", "right", or "middle".
  /// `mode`: "click", "press", or "release".
  #[napi]
  pub fn mouse_click(&mut self, button: String, mode: String) -> Result<()> {
    let btn: MouseButton =
      button
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| {
          napi::Error::from_reason(e.to_string())
        })?;
    let action_mode: ActionMode =
      mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| {
          napi::Error::from_reason(e.to_string())
        })?;
    self
      .inner
      .mouse_click(btn, action_mode)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Move the mouse cursor.
  ///
  /// `mode`: "absolute" or "relative".
  #[napi]
  pub fn mouse_move(&mut self, x: i32, y: i32, mode: String) -> Result<()> {
    let coord_mode: CoordinateMode =
      mode
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| {
          napi::Error::from_reason(e.to_string())
        })?;
    self
      .inner
      .mouse_move(x, y, coord_mode)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Scroll the mouse wheel.
  ///
  /// `axis`: "vertical" or "horizontal".
  #[napi]
  pub fn scroll(&mut self, axis: String, amount: i32) -> Result<()> {
    let scroll_axis: ScrollAxis =
      axis
        .parse()
        .map_err(|e: libmacrocenter::error::MacroCenterError| {
          napi::Error::from_reason(e.to_string())
        })?;
    self
      .inner
      .scroll(scroll_axis, amount)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }
}

/// Global hotkey manager.
#[napi(js_name = "HotkeyManager")]
pub struct JsHotkeyManager {
  inner: HotkeyManager,
}

#[napi]
impl JsHotkeyManager {
  /// Create a new hotkey manager.
  ///
  /// Must be called on a thread with an active event loop.
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let inner = HotkeyManager::new().map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Self { inner })
  }

  /// Register a hotkey from a string like "Ctrl+Shift+M".
  ///
  /// Returns the hotkey ID.
  #[napi]
  pub fn register(&mut self, hotkey: String) -> Result<u32> {
    self
      .inner
      .register(&hotkey)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Unregister a hotkey by ID.
  #[napi]
  pub fn unregister(&mut self, id: u32) -> Result<()> {
    self
      .inner
      .unregister(id)
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Unregister all hotkeys.
  #[napi]
  pub fn unregister_all(&mut self) -> Result<()> {
    self
      .inner
      .unregister_all()
      .map_err(|e| napi::Error::from_reason(e.to_string()))
  }

  /// Poll for a hotkey event (non-blocking).
  ///
  /// Returns null if no event is available.
  #[napi]
  pub fn poll_event(&self) -> Option<JsHotkeyEvent> {
    self.inner.poll_event().map(|e| JsHotkeyEvent {
      id: e.id,
      state: match e.state {
        HotkeyState::Pressed => "pressed".to_string(),
        HotkeyState::Released => "released".to_string(),
      },
    })
  }

  /// Listen for hotkey events using a callback.
  ///
  /// The callback receives standard Node.js arguments: `(err, event)`
  #[napi]
  pub fn listen(
    &self,
    callback: napi::threadsafe_function::ThreadsafeFunction<JsHotkeyEvent>,
  ) -> Result<()> {
    std::thread::spawn(move || {
      HotkeyManager::listen(|event| {
        let js_event = JsHotkeyEvent {
          id: event.id,
          state: match event.state {
            HotkeyState::Pressed => "pressed".to_string(),
            HotkeyState::Released => "released".to_string(),
          },
        };
        callback.call(
          Ok(js_event),
          napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
        );
      });
    });

    Ok(())
  }

  /// Get the list of registered hotkey IDs.
  #[napi]
  pub fn registered_ids(&self) -> Vec<u32> {
    self.inner.registered_ids()
  }
}
