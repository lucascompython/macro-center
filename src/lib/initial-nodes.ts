import type { Node, Edge } from "@xyflow/svelte";
import { ActionMode, CoordinateMode, Axis, MouseButton } from "./types";
import { FLOW_IN_HANDLE, FLOW_OUT_HANDLE } from "./graph";

/**
 * Initial demo nodes showcasing all available node types
 */
export const initialNodes: Node[] = [
  {
    id: "1",
    position: { x: 50, y: 200 },
    type: "keyBindNode",
    data: {
      title: "Key Bind",
      subline: "Trigger macro with key combination",
      shortcut: "Ctrl+Shift+M",
    },
  },
  {
    id: "2",
    position: { x: 350, y: 100 },
    type: "typeNode",
    data: {
      title: "Type Text",
      subline: 'Types "Hello World"',
      text: "Hello World!",
    },
  },
  {
    id: "3",
    position: { x: 650, y: 100 },
    type: "delayNode",
    data: {
      title: "Delay",
      subline: "Wait before continuing",
      delay: 500,
    },
  },
  // Key press node
  {
    id: "4",
    position: { x: 350, y: 350 },
    type: "keyNode",
    data: {
      title: "Key Press",
      subline: "Press/release/click a key",
      key: "Enter",
      mode: ActionMode.CLICK,
    },
  },
  {
    id: "5",
    position: { x: 650, y: 350 },
    type: "mouseMoveNode",
    data: {
      title: "Move Mouse",
      subline: "Move cursor to position",
      x: 500,
      y: 300,
      coordinateMode: CoordinateMode.ABSOLUTE,
    },
  },
  {
    id: "6",
    position: { x: 950, y: 200 },
    type: "mousePressNode",
    data: {
      title: "Mouse Click",
      subline: "Simulate mouse button action",
      button: MouseButton.LEFT,
      mode: ActionMode.CLICK,
    },
  },
  {
    id: "7",
    position: { x: 1250, y: 100 },
    type: "scrollMouseNode",
    data: {
      title: "Scroll Mouse",
      subline: "Scroll the mouse wheel",
      axis: Axis.VERTICAL,
      amount: 3,
    },
  },
];

export const initialEdges: Edge[] = [
  {
    id: "e1-2",
    source: "1",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "2",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e1-4",
    source: "1",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "4",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e2-3",
    source: "2",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "3",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e4-5",
    source: "4",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "5",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e3-6",
    source: "3",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "6",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e5-6",
    source: "5",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "6",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
  {
    id: "e6-7",
    source: "6",
    sourceHandle: FLOW_OUT_HANDLE,
    target: "7",
    targetHandle: FLOW_IN_HANDLE,
    data: { kind: "trigger" },
  },
];
