use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use ::libmacrocenter::error::MacroCenterError;
use ::libmacrocenter::hotkey::{HotkeyManager, HotkeyState};
use ::libmacrocenter::input::InputSimulator;
use ::libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

/// Input simulator for keyboard and mouse actions.
#[pyclass(name = "InputSimulator")]
struct PyInputSimulator {
    inner: InputSimulator,
}

#[pymethods]
impl PyInputSimulator {
    /// Create a new input simulator.
    #[new]
    fn new() -> PyResult<Self> {
        let inner = InputSimulator::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Type a string of text.
    fn type_text(&mut self, text: &str) -> PyResult<()> {
        self.inner
            .type_text(text)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Perform a key action.
    ///
    /// Args:
    ///     key: Key name like "enter", "a", "F5", etc.
    ///     mode: "click", "press", or "release".
    fn key_action(&mut self, key: &str, mode: &str) -> PyResult<()> {
        let action_mode: ActionMode = mode
            .parse()
            .map_err(|e: MacroCenterError| PyRuntimeError::new_err(e.to_string()))?;
        self.inner
            .key_action(key, action_mode)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Perform a mouse button action.
    ///
    /// Args:
    ///     button: "left", "right", or "middle".
    ///     mode: "click", "press", or "release".
    fn mouse_click(&mut self, button: &str, mode: &str) -> PyResult<()> {
        let btn: MouseButton = button
            .parse()
            .map_err(|e: MacroCenterError| PyRuntimeError::new_err(e.to_string()))?;
        let action_mode: ActionMode = mode
            .parse()
            .map_err(|e: MacroCenterError| PyRuntimeError::new_err(e.to_string()))?;
        self.inner
            .mouse_click(btn, action_mode)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Move the mouse cursor.
    ///
    /// Args:
    ///     x: X coordinate.
    ///     y: Y coordinate.
    ///     mode: "absolute" or "relative".
    fn mouse_move(&mut self, x: i32, y: i32, mode: &str) -> PyResult<()> {
        let coord_mode: CoordinateMode = mode
            .parse()
            .map_err(|e: MacroCenterError| PyRuntimeError::new_err(e.to_string()))?;
        self.inner
            .mouse_move(x, y, coord_mode)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Scroll the mouse wheel.
    ///
    /// Args:
    ///     axis: "vertical" or "horizontal".
    ///     amount: Scroll amount (positive or negative).
    fn scroll(&mut self, axis: &str, amount: i32) -> PyResult<()> {
        let scroll_axis: ScrollAxis = axis
            .parse()
            .map_err(|e: MacroCenterError| PyRuntimeError::new_err(e.to_string()))?;
        self.inner
            .scroll(scroll_axis, amount)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

/// Global hotkey manager.
#[pyclass(name = "HotkeyManager", unsendable)]
struct PyHotkeyManager {
    inner: HotkeyManager,
}

#[pymethods]
impl PyHotkeyManager {
    /// Create a new hotkey manager.
    ///
    /// Must be called on a thread with an active event loop.
    #[new]
    fn new() -> PyResult<Self> {
        let inner = HotkeyManager::new().map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { inner })
    }

    /// Register a hotkey from a string like "Ctrl+Shift+M".
    ///
    /// Returns the hotkey ID.
    fn register(&mut self, hotkey: &str) -> PyResult<u32> {
        self.inner
            .register(hotkey)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Unregister a hotkey by ID.
    fn unregister(&mut self, id: u32) -> PyResult<()> {
        self.inner
            .unregister(id)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Unregister all hotkeys.
    fn unregister_all(&mut self) -> PyResult<()> {
        self.inner
            .unregister_all()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    /// Poll for a hotkey event (non-blocking).
    ///
    /// Returns a dict with 'id' and 'state' keys, or None if no event.
    fn poll_event(&self, py: Python<'_>) -> PyResult<Option<Py<pyo3::types::PyDict>>> {
        match self.inner.poll_event() {
            Some(event) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("id", event.id)?;
                dict.set_item(
                    "state",
                    match event.state {
                        HotkeyState::Pressed => "pressed",
                        HotkeyState::Released => "released",
                    },
                )?;
                Ok(Some(dict.unbind()))
            }
            None => Ok(None),
        }
    }

    /// Wait for a single hotkey event, blocking until one arrives or the timeout expires.
    ///
    /// Args:
    ///     timeout: Optional timeout in seconds. Blocks indefinitely if None.
    /// Returns:
    ///     A dict with 'id' and 'state' keys, or None if timed out.
    #[pyo3(signature = (timeout=None))]
    fn wait_event(
        &self,
        py: Python<'_>,
        timeout: Option<f64>,
    ) -> PyResult<Option<Py<pyo3::types::PyDict>>> {
        let duration = timeout.map(std::time::Duration::from_secs_f64);

        let mut result = None;
        let start = std::time::Instant::now();

        loop {
            py.check_signals()?;

            // Allow thread switches and block for a max of 50ms at a time to stay responsive to signals
            let wait_time = if let Some(d) = duration {
                let remaining = d.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    break;
                }
                std::time::Duration::from_millis(50).min(remaining)
            } else {
                std::time::Duration::from_millis(50)
            };

            let event = py.detach(|| HotkeyManager::wait_event(Some(wait_time)));

            if let Some(e) = event {
                result = Some(e);
                break;
            }
        }

        match result {
            Some(event) => {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("id", event.id)?;
                dict.set_item(
                    "state",
                    match event.state {
                        HotkeyState::Pressed => "pressed",
                        HotkeyState::Released => "released",
                    },
                )?;
                Ok(Some(dict.unbind()))
            }
            None => Ok(None),
        }
    }

    /// Listen for hotkey events using a callback.
    ///
    /// The callback receives a single argument: a dict with 'id' and 'state' keys.
    /// Blocks indefinitely, occasionally yielding to the GIL so that signals like Ctrl+C work.
    #[pyo3(signature = (callback))]
    fn listen(&self, py: Python<'_>, callback: Py<PyAny>) -> PyResult<()> {
        loop {
            py.check_signals()?;

            let event =
                py.detach(|| HotkeyManager::wait_event(Some(std::time::Duration::from_millis(50))));

            if let Some(event) = event {
                let dict = pyo3::types::PyDict::new(py);
                dict.set_item("id", event.id)?;
                dict.set_item(
                    "state",
                    match event.state {
                        HotkeyState::Pressed => "pressed",
                        HotkeyState::Released => "released",
                    },
                )?;
                callback.call1(py, (dict,))?;
            }
        }
    }

    /// Get the list of registered hotkey IDs.
    fn registered_ids(&self) -> Vec<u32> {
        self.inner.registered_ids()
    }
}

/// A Python module implemented in Rust.
#[pymodule]
mod macrocenter_python {
    #[pymodule_export]
    use super::PyInputSimulator;

    #[pymodule_export]
    use super::PyHotkeyManager;
}
