export enum ActionMode {
  CLICK = "click",
  PRESS = "press",
  RELEASE = "release",
}

export enum CoordinateMode {
  ABSOLUTE = "absolute",
  RELATIVE = "relative",
}

export enum Axis {
  VERTICAL = "vertical",
  HORIZONTAL = "horizontal",
}

export enum MouseButton {
  LEFT = "left",
  RIGHT = "right",
  MIDDLE = "middle",
  MOUSE4 = "mouse4",
  MOUSE5 = "mouse5",
}

export interface BaseNodeData {
  title: string;
  subline?: string;
  [key: string]: unknown;
}

export interface KeyBindNodeData extends BaseNodeData {
  keyBind: string;
}

export interface TypeNodeData extends BaseNodeData {
  text: string;
  delay: number;
  cancelKey?: string;
}

export interface KeyNodeData extends BaseNodeData {
  key: string;
  mode: ActionMode;
  delay?: number;
  cancelKey?: string;
}

export interface MousePressNodeData extends BaseNodeData {
  button: MouseButton;
  mode: ActionMode;
}

export interface MouseMoveNodeData extends BaseNodeData {
  x: number;
  y: number;
  coordinateMode: CoordinateMode;
}

export interface DelayNodeData extends BaseNodeData {
  delay: number;
}

export interface ScrollMouseNodeData extends BaseNodeData {
  axis: Axis;
  amount: number;
}

export interface ConditionalNodeData extends BaseNodeData {
  condition: string;
}
