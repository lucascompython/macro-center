use mlua::prelude::*;

use libmacrocenter::hotkey::{HotkeyManager, HotkeyState};
use libmacrocenter::input::InputSimulator;
use libmacrocenter::types::{ActionMode, CoordinateMode, MouseButton, ScrollAxis};

/// Wraps `InputSimulator` as Lua userdata.
struct LuaInputSimulator(InputSimulator);

impl LuaUserData for LuaInputSimulator {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("type_text", |_, this, text: String| {
            this.0
                .type_text(&text)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method_mut("key_action", |_, this, (key, mode): (String, String)| {
            let action_mode: ActionMode = mode
                .parse()
                .map_err(|e: libmacrocenter::error::MacroCenterError| {
                    LuaError::RuntimeError(e.to_string())
                })?;
            this.0
                .key_action(&key, action_mode)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method_mut(
            "mouse_click",
            |_, this, (button, mode): (String, String)| {
                let btn: MouseButton = button
                    .parse()
                    .map_err(|e: libmacrocenter::error::MacroCenterError| {
                        LuaError::RuntimeError(e.to_string())
                    })?;
                let action_mode: ActionMode = mode
                    .parse()
                    .map_err(|e: libmacrocenter::error::MacroCenterError| {
                        LuaError::RuntimeError(e.to_string())
                    })?;
                this.0
                    .mouse_click(btn, action_mode)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))
            },
        );

        methods.add_method_mut(
            "mouse_move",
            |_, this, (x, y, mode): (i32, i32, String)| {
                let coord_mode: CoordinateMode = mode
                    .parse()
                    .map_err(|e: libmacrocenter::error::MacroCenterError| {
                        LuaError::RuntimeError(e.to_string())
                    })?;
                this.0
                    .mouse_move(x, y, coord_mode)
                    .map_err(|e| LuaError::RuntimeError(e.to_string()))
            },
        );

        methods.add_method_mut("scroll", |_, this, (axis, amount): (String, i32)| {
            let scroll_axis: ScrollAxis = axis
                .parse()
                .map_err(|e: libmacrocenter::error::MacroCenterError| {
                    LuaError::RuntimeError(e.to_string())
                })?;
            this.0
                .scroll(scroll_axis, amount)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });
    }
}

/// Wraps `HotkeyManager` as Lua userdata.
struct LuaHotkeyManager(HotkeyManager);

impl LuaUserData for LuaHotkeyManager {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("register", |_, this, hotkey: String| {
            this.0
                .register(&hotkey)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method_mut("unregister", |_, this, id: u32| {
            this.0
                .unregister(id)
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method_mut("unregister_all", |_, this, (): ()| {
            this.0
                .unregister_all()
                .map_err(|e| LuaError::RuntimeError(e.to_string()))
        });

        methods.add_method("poll_event", |lua, this, (): ()| {
            match this.0.poll_event() {
                Some(event) => {
                    let table = lua.create_table()?;
                    table.set("id", event.id)?;
                    table.set(
                        "state",
                        match event.state {
                            HotkeyState::Pressed => "pressed",
                            HotkeyState::Released => "released",
                        },
                    )?;
                    Ok(LuaValue::Table(table))
                }
                None => Ok(LuaValue::Nil),
            }
        });

        methods.add_method("wait_event", |lua, _, timeout_secs: Option<f64>| {
            let duration = timeout_secs.map(std::time::Duration::from_secs_f64);
            match HotkeyManager::wait_event(duration) {
                Some(event) => {
                    let table = lua.create_table()?;
                    table.set("id", event.id)?;
                    table.set(
                        "state",
                        match event.state {
                            HotkeyState::Pressed => "pressed",
                            HotkeyState::Released => "released",
                        },
                    )?;
                    Ok(LuaValue::Table(table))
                }
                None => Ok(LuaValue::Nil),
            }
        });

        methods.add_method_mut("listen", |lua, _, callback: mlua::Function| {
            // Blocks the current Lua thread indefinitely loop-calling the callback.
            HotkeyManager::listen(|event| {
                let table = lua.create_table().unwrap();
                table.set("id", event.id).unwrap();
                table.set(
                    "state",
                    match event.state {
                        HotkeyState::Pressed => "pressed",
                        HotkeyState::Released => "released",
                    },
                ).unwrap();
                let _: () = callback.call(table).unwrap();
            });
            Ok(())
        });

        methods.add_method("registered_ids", |lua, this, (): ()| {
            let ids = this.0.registered_ids();
            let table = lua.create_table()?;
            for (i, id) in ids.iter().enumerate() {
                table.set(i + 1, *id)?;
            }
            Ok(table)
        });
    }
}

/// Module entrypoint for Lua.
#[mlua::lua_module]
fn macrocenter_lua(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    exports.set(
        "InputSimulator",
        lua.create_function(|_, (): ()| {
            let sim = InputSimulator::new()
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            Ok(LuaInputSimulator(sim))
        })?,
    )?;

    exports.set(
        "HotkeyManager",
        lua.create_function(|_, (): ()| {
            let mgr = HotkeyManager::new()
                .map_err(|e| LuaError::RuntimeError(e.to_string()))?;
            Ok(LuaHotkeyManager(mgr))
        })?,
    )?;

    Ok(exports)
}
