# Macro Center a Visual Programming tool for Macros

Macro Center is a cross-platform tauri application designed to simplify the creation and management of macros through a visual programming interface (node-based UI). It allows users to automate repetitive tasks by creating macros using a simple to use interface, making it accessible for users without programming experience.

## Features

- Node-Based Visual Programming: Users can create macros by connecting nodes that represent different actions and logic.
  - This is done using the svelteflow library (xyflow, svelte version, @xyflow/svelte)
  - Will use tailwind where possible for styling, for costumizing xyflow nodes, im not sure you can use tailwind directly, so might need to use plain old css for that.
- Predefined Nodes: The application comes with a variety of predefined nodes for common actions such as keyboard input, mouse movements, delays, and conditional logic, for example:
  - keybind node: to bind a macro to a specific key combination.
    - Inputs: none
    - Outputs: trigger
    - Parameters: key combination (e.g., Ctrl+Shift+M or F5)
  - type node: types a given string.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters: string to type
  - key node: presses a given key.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters:
      - key to press (e.g., 'A', 'Enter', 'Ctrl')
      - mode: 'press', 'release' and 'click' (press and release)
  - mouse press node: simulates mouse button actions.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters:
      - button: 'left', 'right', 'middle', "mouse4", "mouse5", etc.
      - mode: 'press', 'release', 'click'
  - mouse move node: moves the mouse cursor to specified coordinates.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters:
      - x coordinate
      - y coordinate
      - mode: 'absolute' or 'relative'
  - delay node: introduces a delay in the macro execution.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters: delay time in milliseconds
  - scroll mouse node: scrolls the mouse wheel.
    - Inputs: trigger
    - Outputs: trigger
    - Parameters:
      - amount: positive or negative integer for scroll direction
  - conditional node: executes different branches based on a condition.
    - Inputs: trigger, condition input
    - Outputs: true branch trigger, false branch trigger
    - Parameters: condition expression (e.g., variable comparison)
  - etc.

- Cross-Platform Support: Built using Tauri, Macro Center runs on Windows, macOS, and Linux.
  - The listening of the events is done via the [tauri-plugin-global-shortcut](https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/global-shortcut) plugin.
  - The mouse and keyboard events are simulated using the [enigo](https://github.com/enigo-rs/enigo) crate.

- Macro Management: Users can save, load, and organize their macros within the application. (probably save them as json files)

## For reference

I had previously worked on a similar project called Auto-Spammer, but never got far with it, it implemented some of nodes UI. I will use it as a reference for this project.

## For the Future

- Page/mode to record user actions and convert them into macros.
- Create libmacrocenter, a rust library that can be used to listen to global shortcuts and simulate mouse and keyboard events, so that other applications/languages can use it as a dependency.

## TODO (mostly non urgent):

- Make it so you can create nodes by clicking on the canvas or grabbing an edge and dropping it somewhere, not only from the sidebar.
- Fix horrible input UX in nodes.
- For key bind node, make it so you can record keybinds instead of typing them.
