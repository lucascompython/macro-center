import type { Node, Edge } from "@xyflow/svelte";
import { ActionMode, CoordinateMode, Axis, MouseButton } from "./types";

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
      keyBind: "Ctrl+Shift+M",
    },
  },
  {
    id: "2",
    position: { x: 350, y: 100 },
    type: "typeNode",
    data: {
      title: "Type Text",
      subline: "Types a given text",
      text: "Hello World!",
      delay: 0.5,
      cancelKey: "",
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
      delay: 0.1,
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
  // Conditional node
  {
    id: "8",
    position: { x: 1250, y: 350 },
    type: "conditionalNode",
    data: {
      title: "Conditional",
      subline: "Branch based on condition",
      condition: "count > 5",
    },
  },
];

export const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e1-4", source: "1", target: "4" },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e4-5", source: "4", target: "5" },
  { id: "e3-6", source: "3", target: "6" },
  { id: "e5-6", source: "5", target: "6" },
  { id: "e6-7", source: "6", target: "7" },
  { id: "e6-8", source: "6", target: "8" },
];
