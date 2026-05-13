import type { Edge, Node } from "@xyflow/svelte";

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
  shortcut: string;
}

export interface TypeNodeData extends BaseNodeData {
  text: string;
  delay: number;
}

export interface KeyNodeData extends BaseNodeData {
  key: string;
  mode: ActionMode;
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
  conditionExpression?: ConditionExpression;
}

export type ValueType = "boolean" | "number" | "text" | "key" | "mouseButton" | "point" | "list";

export type VariableScope = "macro" | "local" | "persistent";

export interface PointValue {
  x: number;
  y: number;
}

export type MacroValue = string | number | boolean | PointValue | MacroValue[] | null;

export interface VariableDefinition {
  id: string;
  name: string;
  type: ValueType;
  defaultValue: MacroValue;
  scope: VariableScope;
}

export type OperandSource = "literal" | "variable";

export interface ValueOperand {
  source: OperandSource;
  value?: MacroValue;
  variableName?: string;
}

export type ConditionOperator =
  | "truthy"
  | "equals"
  | "notEquals"
  | "greaterThan"
  | "greaterThanOrEqual"
  | "lessThan"
  | "lessThanOrEqual"
  | "contains"
  | "startsWith"
  | "endsWith";

export interface ConditionExpression {
  left: ValueOperand;
  operator: ConditionOperator;
  right?: ValueOperand;
}

export interface SetVariableNodeData extends BaseNodeData {
  variableName: string;
  valueType: ValueType;
  value: MacroValue;
  scope?: VariableScope;
}

export interface GetVariableNodeData extends BaseNodeData {
  variableName: string;
}

export type UpdateVariableOperation =
  | "increment"
  | "decrement"
  | "set"
  | "append"
  | "toggle"
  | "clear";

export interface UpdateVariableNodeData extends BaseNodeData {
  variableName: string;
  operation: UpdateVariableOperation;
  value: MacroValue;
}

export interface CompareNodeData extends BaseNodeData {
  conditionExpression: ConditionExpression;
}

export interface ValueNodeData extends BaseNodeData {
  valueType: ValueType;
  value: MacroValue;
}

export interface RepeatLoopNodeData extends BaseNodeData {
  iterations: number;
  indexVariable: string;
}

export interface ForEachLoopNodeData extends BaseNodeData {
  items: MacroValue[];
  itemVariable: string;
  indexVariable: string;
}

export interface WhileLoopNodeData extends BaseNodeData {
  conditionExpression: ConditionExpression;
  indexVariable: string;
  maxIterations: number;
}

export interface SubmacroTriggerOutput {
  id: string;
  label: string;
}

export interface SubmacroValuePort {
  id: string;
  label: string;
  type: ValueType;
  defaultValue?: MacroValue;
}

export interface SubmacroBoundaryOutput {
  outputId: string;
  sourceNodeId: string;
  sourceHandle?: string | null;
}

export interface SubmacroBoundaryValueInput {
  inputId: string;
  targetNodeId: string;
  targetHandle?: string | null;
}

export interface SubmacroValueOutputBinding {
  outputId: string;
  sourceNodeId: string;
  sourceHandle?: string | null;
}

export interface SubmacroDefinition {
  id: string;
  name: string;
  nodes: Node[];
  edges: Edge[];
  variables: VariableDefinition[];
  entryNodeIds: string[];
  triggerOutputs: SubmacroTriggerOutput[];
  valueInputs: SubmacroValuePort[];
  valueOutputs: SubmacroValuePort[];
  boundaryValueInputs: SubmacroBoundaryValueInput[];
  valueOutputBindings: SubmacroValueOutputBinding[];
  boundaryOutputs: SubmacroBoundaryOutput[];
  createdAt: string;
  updatedAt: string;
}

export interface SubmacroNodeData extends BaseNodeData {
  definitionId: string;
  triggerOutputs: SubmacroTriggerOutput[];
  valueInputs: SubmacroValuePort[];
  valueOutputs: SubmacroValuePort[];
}

export interface BreakLoopNodeData extends BaseNodeData {}

export interface ContinueLoopNodeData extends BaseNodeData {}

export interface MacroProjectData {
  nodes: Node[];
  edges: Edge[];
  variables: VariableDefinition[];
  submacros: SubmacroDefinition[];
  viewport?: {
    x: number;
    y: number;
    zoom: number;
  };
}
