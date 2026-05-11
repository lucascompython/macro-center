export const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

export function keyToAccelerator(key: string): string {
  const map: Record<string, string> = {
    Control: "Ctrl",
    Meta: "Super",
    " ": "Space",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Escape: "Escape",
    Enter: "Enter",
    Backspace: "Backspace",
    Delete: "Delete",
    Tab: "Tab",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
    Insert: "Insert",
  };

  if (map[key]) return map[key];
  if (/^F\d{1,2}$/.test(key)) return key;
  if (key.length === 1) return key.toUpperCase();
  return key;
}

export function modifierShortcutPreview(event: KeyboardEvent): string {
  const parts = shortcutModifiers(event);
  return parts.length > 0 ? `${parts.join("+")}+...` : "...";
}

export function shortcutFromKeyboardEvent(event: KeyboardEvent): string | undefined {
  if (MODIFIER_KEYS.has(event.key)) return undefined;

  const parts = shortcutModifiers(event);
  parts.push(keyToAccelerator(event.key));
  return parts.join("+");
}

function shortcutModifiers(event: KeyboardEvent) {
  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.shiftKey) parts.push("Shift");
  if (event.altKey) parts.push("Alt");
  if (event.metaKey) parts.push("Super");
  return parts;
}
