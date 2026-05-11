#[cfg(feature = "rapidhash")]
use rapidhash::HashMapExt;
#[cfg(feature = "rapidhash")]
use rapidhash::RapidHashMap as HashMap;

#[cfg(not(feature = "rapidhash"))]
use std::collections::HashMap;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

use crate::error::{MacroCenterError, Result};

/// State of a hotkey event.
#[derive(Debug)]
pub enum HotkeyState {
    Pressed,
    Released,
}

/// A hotkey event received from the global hotkey system.
#[derive(Debug)]
pub struct HotkeyEvent {
    /// The ID of the hotkey that was triggered.
    pub id: u32,
    /// The state of the hotkey (pressed or released).
    pub state: HotkeyState,
}

/// Backend abstraction for registering and receiving global hotkeys.
pub trait HotKeyBackend {
    fn register(&mut self, hotkey_str: &str) -> Result<u32>;
    fn unregister(&mut self, id: u32) -> Result<()>;
    fn unregister_all(&mut self) -> Result<()>;
    fn poll_event(&self) -> Option<HotkeyEvent>;
    fn registered_ids(&self) -> Vec<u32>;
}

/// Global hotkey backend backed by the same global-hotkey APIs used by Tauri.
///
/// # Platform Requirements
///
/// - **Windows**: A win32 event loop must be running on the same thread.
/// - **macOS**: An event loop must be running on the main thread.
pub struct TauriGlobalHotkey {
    manager: GlobalHotKeyManager,
    registered: HashMap<u32, HotKey>,
}

/// Backwards-compatible name for the default hotkey manager.
pub type HotkeyManager = TauriGlobalHotkey;

impl TauriGlobalHotkey {
    /// Create a new hotkey manager.
    ///
    /// Must be called on a thread with an active event loop (see platform requirements).
    pub fn new() -> Result<Self> {
        let manager =
            GlobalHotKeyManager::new().map_err(|e| MacroCenterError::HotkeyError(e.to_string()))?;
        Ok(Self {
            manager,
            registered: HashMap::new(),
        })
    }

    /// Register a hotkey from a human-readable string like `"Ctrl+Shift+M"` or `"F5"`.
    ///
    /// Returns the hotkey ID which can be used to identify events and unregister.
    pub fn register(&mut self, hotkey_str: &str) -> Result<u32> {
        let hotkey = parse_hotkey(hotkey_str)?;
        self.manager
            .register(hotkey)
            .map_err(|e| MacroCenterError::HotkeyError(e.to_string()))?;
        let id = hotkey.id();
        self.registered.insert(id, hotkey);
        Ok(id)
    }

    /// Unregister a hotkey by its ID.
    pub fn unregister(&mut self, id: u32) -> Result<()> {
        if let Some(hotkey) = self.registered.remove(&id) {
            self.manager
                .unregister(hotkey)
                .map_err(|e| MacroCenterError::HotkeyError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn unregister_all(&mut self) -> Result<()> {
        let hotkeys: Vec<HotKey> = self.registered.drain().map(|(_, v)| v).collect();
        for hotkey in hotkeys {
            // TODO: Probably should handle errors here
            self.manager.unregister(hotkey).unwrap();
        }
        Ok(())
    }

    /// Non-blocking poll for hotkey events.
    ///
    /// Returns `Some(HotkeyEvent)` if an event is available, `None` otherwise.
    pub fn poll_event(&self) -> Option<HotkeyEvent> {
        GlobalHotKeyEvent::receiver().try_recv().ok().map(|event| {
            let state = match event.state() {
                global_hotkey::HotKeyState::Pressed => HotkeyState::Pressed,
                global_hotkey::HotKeyState::Released => HotkeyState::Released,
            };
            HotkeyEvent {
                id: event.id(),
                state,
            }
        })
    }

    /// Get the list of currently registered hotkey IDs.
    pub fn registered_ids(&self) -> Vec<u32> {
        self.registered.keys().copied().collect()
    }

    /// Listen to all global hotkey events indefinitely.
    ///
    /// This blocks the current thread and calls the provided callback for each event.
    pub fn listen<F>(mut callback: F)
    where
        F: FnMut(HotkeyEvent),
    {
        let receiver = GlobalHotKeyEvent::receiver();
        while let Ok(event) = receiver.recv() {
            let state = match event.state() {
                global_hotkey::HotKeyState::Pressed => HotkeyState::Pressed,
                global_hotkey::HotKeyState::Released => HotkeyState::Released,
            };
            callback(HotkeyEvent {
                id: event.id(),
                state,
            });
        }
    }

    /// Wait for a single hotkey event, blocking until one arrives or the timeout expires.
    ///
    /// If `timeout` is `None`, this blocks indefinitely.
    pub fn wait_event(timeout: Option<std::time::Duration>) -> Option<HotkeyEvent> {
        let receiver = GlobalHotKeyEvent::receiver();
        let event_result = match timeout {
            Some(t) => receiver.recv_timeout(t).ok(),
            None => receiver.recv().ok(),
        };

        event_result.map(|event| {
            let state = match event.state() {
                global_hotkey::HotKeyState::Pressed => HotkeyState::Pressed,
                global_hotkey::HotKeyState::Released => HotkeyState::Released,
            };
            HotkeyEvent {
                id: event.id(),
                state,
            }
        })
    }
}

impl HotKeyBackend for TauriGlobalHotkey {
    fn register(&mut self, hotkey_str: &str) -> Result<u32> {
        TauriGlobalHotkey::register(self, hotkey_str)
    }

    fn unregister(&mut self, id: u32) -> Result<()> {
        TauriGlobalHotkey::unregister(self, id)
    }

    fn unregister_all(&mut self) -> Result<()> {
        TauriGlobalHotkey::unregister_all(self)
    }

    fn poll_event(&self) -> Option<HotkeyEvent> {
        TauriGlobalHotkey::poll_event(self)
    }

    fn registered_ids(&self) -> Vec<u32> {
        TauriGlobalHotkey::registered_ids(self)
    }
}

/// Parse a human-readable hotkey string into a `HotKey`.
///
/// Format: `"Modifier+Modifier+Key"` where modifiers are optional.
/// Examples: `"Ctrl+Shift+M"`, `"Alt+F5"`, `"F12"`, `"Ctrl+A"`
///
/// Supported modifiers: `Ctrl`/`Control`, `Shift`, `Alt`, `Super`/`Meta`/`Win`/`Cmd`
fn parse_hotkey(s: &str) -> Result<HotKey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

    if parts.is_empty() {
        return Err(MacroCenterError::ParseError(
            "Empty hotkey string".to_string(),
        ));
    }

    let mut modifiers = Modifiers::empty();
    let mut key_part = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "super" | "meta" | "win" | "cmd" => modifiers |= Modifiers::META,
            _ => {
                if key_part.is_some() {
                    return Err(MacroCenterError::ParseError(format!(
                        "Multiple key codes in hotkey string: '{s}'"
                    )));
                }
                key_part = Some(*part);
            }
        }
    }

    let key_str = key_part.ok_or_else(|| {
        MacroCenterError::ParseError(format!("No key code found in hotkey string: '{s}'"))
    })?;

    let code = parse_code(key_str)?;

    let mods = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    };

    Ok(HotKey::new(mods, code))
}

/// Parse a key name string into a `global_hotkey::hotkey::Code`.
fn parse_code(s: &str) -> Result<Code> {
    match s.to_lowercase().as_str() {
        // Letters
        "a" => Ok(Code::KeyA),
        "b" => Ok(Code::KeyB),
        "c" => Ok(Code::KeyC),
        "d" => Ok(Code::KeyD),
        "e" => Ok(Code::KeyE),
        "f" => Ok(Code::KeyF),
        "g" => Ok(Code::KeyG),
        "h" => Ok(Code::KeyH),
        "i" => Ok(Code::KeyI),
        "j" => Ok(Code::KeyJ),
        "k" => Ok(Code::KeyK),
        "l" => Ok(Code::KeyL),
        "m" => Ok(Code::KeyM),
        "n" => Ok(Code::KeyN),
        "o" => Ok(Code::KeyO),
        "p" => Ok(Code::KeyP),
        "q" => Ok(Code::KeyQ),
        "r" => Ok(Code::KeyR),
        "s" => Ok(Code::KeyS),
        "t" => Ok(Code::KeyT),
        "u" => Ok(Code::KeyU),
        "v" => Ok(Code::KeyV),
        "w" => Ok(Code::KeyW),
        "x" => Ok(Code::KeyX),
        "y" => Ok(Code::KeyY),
        "z" => Ok(Code::KeyZ),
        // Digits
        "0" => Ok(Code::Digit0),
        "1" => Ok(Code::Digit1),
        "2" => Ok(Code::Digit2),
        "3" => Ok(Code::Digit3),
        "4" => Ok(Code::Digit4),
        "5" => Ok(Code::Digit5),
        "6" => Ok(Code::Digit6),
        "7" => Ok(Code::Digit7),
        "8" => Ok(Code::Digit8),
        "9" => Ok(Code::Digit9),
        // Function keys
        "f1" => Ok(Code::F1),
        "f2" => Ok(Code::F2),
        "f3" => Ok(Code::F3),
        "f4" => Ok(Code::F4),
        "f5" => Ok(Code::F5),
        "f6" => Ok(Code::F6),
        "f7" => Ok(Code::F7),
        "f8" => Ok(Code::F8),
        "f9" => Ok(Code::F9),
        "f10" => Ok(Code::F10),
        "f11" => Ok(Code::F11),
        "f12" => Ok(Code::F12),
        // Special keys
        "space" => Ok(Code::Space),
        "enter" | "return" => Ok(Code::Enter),
        "tab" => Ok(Code::Tab),
        "backspace" => Ok(Code::Backspace),
        "escape" | "esc" => Ok(Code::Escape),
        "delete" | "del" => Ok(Code::Delete),
        "insert" => Ok(Code::Insert),
        "home" => Ok(Code::Home),
        "end" => Ok(Code::End),
        "pageup" => Ok(Code::PageUp),
        "pagedown" => Ok(Code::PageDown),
        // Arrow keys
        "up" | "arrowup" => Ok(Code::ArrowUp),
        "down" | "arrowdown" => Ok(Code::ArrowDown),
        "left" | "arrowleft" => Ok(Code::ArrowLeft),
        "right" | "arrowright" => Ok(Code::ArrowRight),
        // Punctuation / symbols
        "minus" | "-" => Ok(Code::Minus),
        "equal" | "=" => Ok(Code::Equal),
        "bracketleft" | "[" => Ok(Code::BracketLeft),
        "bracketright" | "]" => Ok(Code::BracketRight),
        "backslash" | "\\" => Ok(Code::Backslash),
        "semicolon" | ";" => Ok(Code::Semicolon),
        "quote" | "'" => Ok(Code::Quote),
        "backquote" | "`" => Ok(Code::Backquote),
        "comma" | "," => Ok(Code::Comma),
        "period" | "." => Ok(Code::Period),
        "slash" | "/" => Ok(Code::Slash),
        _ => Err(MacroCenterError::ParseError(format!(
            "Unknown key code: '{s}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_key_hotkey() {
        let hk = parse_hotkey("F5").unwrap();
        assert_eq!(hk.id(), HotKey::new(None, Code::F5).id());
    }

    #[test]
    fn parse_modifier_key_hotkey() {
        let hk = parse_hotkey("Ctrl+A").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL), Code::KeyA);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn parse_multi_modifier_hotkey() {
        let hk = parse_hotkey("Ctrl+Shift+M").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn parse_hotkey_case_insensitive() {
        let hk = parse_hotkey("ctrl+shift+m").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyM);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn parse_hotkey_with_spaces() {
        let hk = parse_hotkey("Ctrl + A").unwrap();
        let expected = HotKey::new(Some(Modifiers::CONTROL), Code::KeyA);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn parse_empty_hotkey_fails() {
        assert!(parse_hotkey("").is_err());
    }

    #[test]
    fn parse_modifier_only_fails() {
        assert!(parse_hotkey("Ctrl+Shift").is_err());
    }

    #[test]
    fn parse_unknown_key_fails() {
        assert!(parse_hotkey("Ctrl+UnknownKey").is_err());
    }

    #[test]
    fn parse_code_letters() {
        assert!(matches!(parse_code("a").unwrap(), Code::KeyA));
        assert!(matches!(parse_code("Z").unwrap(), Code::KeyZ));
    }

    #[test]
    fn parse_code_digits() {
        assert!(matches!(parse_code("0").unwrap(), Code::Digit0));
        assert!(matches!(parse_code("9").unwrap(), Code::Digit9));
    }

    #[test]
    fn parse_code_special() {
        assert!(matches!(parse_code("space").unwrap(), Code::Space));
        assert!(matches!(parse_code("enter").unwrap(), Code::Enter));
        assert!(matches!(parse_code("esc").unwrap(), Code::Escape));
    }
}
