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

## Logic and Reusable Node Groups

Macro Center supports real programming concepts without making the user write code. The logic system should feel like building a flowchart with small, readable blocks instead of wiring a pile of low-level nodes.

### Graph model

- Use two kinds of connections:
  - Trigger flow connections: control the order of execution. These are the "do this, then this" links between action nodes.
  - Value connections: pass data into node parameters. These are used for strings, numbers, booleans, coordinates, lists, etc.

### Variables

- a Variables panel where users can create, rename, inspect, and delete variables used by the current macro.
- Variables should have simple types:
  - boolean
  - number
  - text
  - key
  - mouse button
  - point, with x and y
  - list
- Support a few scopes:
  - Macro variables: shared across one macro run.
  - Submacro local variables: private to a reusable node group.
  - Persistent variables: optional saved values, useful for counters or user preferences between runs.
- Node parameters can reference variables using readable chips, for example `message`, `click_count`, or `target_position`.
- Useful variable nodes:
  - Set Variable: assigns a value.
  - Get Variable: outputs the current value.
  - Update Variable: increments, decrements, appends text, toggles a boolean, etc.
  - Clear Variable: resets a variable to its default value.
- Show live previews while testing a macro, so the user can see the current value of each variable.

### Conditions

- an If node for normal branching.
  - Inputs: trigger, condition
  - Outputs: true trigger, false trigger
  - Parameters: optional condition builder when no condition input is connected
- The condition builder should be UI-first instead of raw text-first:
  - left value: literal, variable, or value input
  - operator: equals, not equals, greater than, contains, is pressed, exists, etc.
  - right value: literal, variable, or value input when needed
- a Switch node for multiple branches from one value, useful when a variable can have several known states.
- small pure value nodes for reusable expressions:
  - Compare
  - And / Or / Not
  - Math
  - Text contains / starts with / ends with
  - Mouse position
  - Key state
- Pure value nodes do not execute actions by themselves. They only calculate values when an execution node needs them.

### Loops

- loop nodes that keep common cases easy:
  - Repeat N: runs a body a fixed number of times.
  - While: runs while a condition is true.
  - For Each: runs once for each item in a list.
- A loop should create a visible loop frame/body on the canvas instead of requiring a messy edge that points backward.
- Loop outputs:
  - body trigger: runs the loop body.
  - done trigger: continues after the loop is complete.
  - break input: exits the loop early.
  - continue input: skips to the next iteration.
- Loop variables should be automatic:
  - `index` for Repeat N and While.
  - `item` and `index` for For Each.
- Add a safety limit for While loops to prevent infinite loops, with a clear setting like "max iterations".

### Visible Subflows and Submacro calls

- Let users select a group of nodes and choose "Create Subflow".
- A subflow is a visible, editable graph definition that stays on the canvas inside a labeled group.
- A Submacro call node is a compact single node that runs one of those subflow definitions.
- When creating a subflow, Macro Center should detect edges that cross the selection boundary and turn them into inputs and outputs on the call node.
- Submacros should support:
  - trigger inputs, like `run`
  - trigger outputs, like `done`, `success`, or `failed`
  - value inputs, like `text`, `delay_ms`, or `position`
  - value outputs, like `result`, `count`, or `found`
  - local variables that do not leak into the parent macro
- The visible subflow should show its name and boundary ports, and its internal nodes should remain editable.
- The visible subflow name should be editable directly from the group header, and renaming it should update all matching Submacro call nodes.
- The compact Submacro call node should show its name, inputs, outputs, and the most important parameters.
- Editing a visible subflow definition should update all places where it is used.
- Support duplicating a submacro into a separate copy when the user wants to change one instance without changing the original.
- Store submacros in a library/sidebar so users can reuse them across macros.
- Prevent accidental recursion at first. A submacro should not be able to directly or indirectly call itself until there is a deliberate recursion design with safety limits.

### Example workflow

1. User creates a macro that clicks a button, waits, checks a condition, and retries up to 5 times.
2. User selects those nodes and chooses "Create Subflow".
3. Macro Center keeps those nodes visible inside a subflow named `Retry Click`.
4. The subflow exposes value inputs like `position`, `attempts`, and `delay_ms`.
5. The parent macro can call it with one `Retry Click` Submacro node instead of duplicating the node chain.

## For reference

I had previously worked on a similar project called Auto-Spammer, but never got far with it, it implemented some of nodes UI. I will use it as a reference for this project.

## TODO:

- Fix the Keybind node to work with numpad keys
- Fix the Keybind node to work with mouse buttons

- Create C header
- See about integrating boltffi
- see why bun and deno don't really work
- see why we need to sleep before typing text (atleast on x11 linux)
- publish packages to crates.io, pypi, npm and luarocks with proper CI
- update rust example
- add a cargo profile for the macrocenter app and another for the lib

- Add a Comment node or a way for users to add comments for users to add notes and explanations to their macros, this would be especially useful for complex macros or for sharing macros with others.
- Add CLI support to run macros from the command line, this would allow users to integrate macros into their existing workflows and automate tasks without needing to open the Macro Center application.
- Make macro files .mc and associate them with Macro Center
- Scheduler node that can run macros at specific times
- Image, OCR and Pixel change recognition
- Add a start menu
- Add settings page

- Fix bug where when in cycles (while, repeat n, for each) nodes, when pressing stop it doesnt actually stop
- Add a way to toogle macro execution
- The window is closed stop execution
- Fix bug where pasting copied node selects also original node
- Fix bug at least on linux where saving a macro doesnt save the extension so the user has to add it manually, it should like this: if there isnt an extension, add .mc, if there is an extension but it isnt .mc, append .mc to it, if there is an extension and it is .mc, do nothing
- Remove frontend tauri plugins where we dont use them in the frontend
- Optimize the logic for conditionals
