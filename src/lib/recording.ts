import type { Edge, Node } from "@xyflow/svelte";
import { ActionMode, Axis, CoordinateMode, MouseButton } from "./types";
import { FLOW_IN_HANDLE, FLOW_OUT_HANDLE } from "./graph";

export type RecordingMouseMode = "movesBeforeClicks" | "allMoves";

export interface RecordedMacro {
  actions: RecordedAction[];
}

export interface RecordedAction {
  delayMs: number;
  kind: RecordedActionKind;
}

export type RecordedActionKind =
  | { type: "key"; key: string; mode: ActionMode }
  | { type: "mouseButton"; button: MouseButton; mode: ActionMode }
  | {
      type: "mouseMove";
      x: number;
      y: number;
      coordinateMode: CoordinateMode;
    }
  | { type: "scroll"; axis: Axis; amount: number };

export interface RecordedGraph {
  nodes: Node[];
  edges: Edge[];
}

const NODE_X_GAP = 380;
const NODE_Y_GAP = 190;
const NODES_PER_ROW = 5;
const MIN_DELAY_MS = 5;

export function recordedMacroToGraph(
  recorded: RecordedMacro,
  shortcut: string,
  origin: { x: number; y: number },
): RecordedGraph {
  const nodes: Node[] = [];
  const edges: Edge[] = [];
  let previousNodeId: string | undefined;
  let index = 0;

  const pushNode = (type: string, data: Record<string, unknown>) => {
    const id = crypto.randomUUID();
    const row = Math.floor(index / NODES_PER_ROW);
    const column = index % NODES_PER_ROW;
    index += 1;

    nodes.push({
      id,
      type,
      position: {
        x: origin.x + column * NODE_X_GAP,
        y: origin.y + row * NODE_Y_GAP,
      },
      data,
    });

    if (previousNodeId) {
      edges.push(makeRecordedEdge(previousNodeId, id));
    }
    previousNodeId = id;
    return id;
  };

  pushNode("keyBindNode", {
    title: "Recorded Macro",
    subline: "Recorded trigger",
    shortcut,
  });

  for (const [actionIndex, action] of recorded.actions.entries()) {
    if (actionIndex > 0 && action.delayMs >= MIN_DELAY_MS) {
      pushNode("delayNode", {
        title: "Delay",
        subline: "Recorded pause",
        delay: Math.round(action.delayMs),
      });
    }

    pushRecordedActionNode(pushNode, action.kind);
  }

  return { nodes, edges };
}

function pushRecordedActionNode(
  pushNode: (type: string, data: Record<string, unknown>) => string,
  kind: RecordedActionKind,
) {
  switch (kind.type) {
    case "key":
      pushNode("keyNode", {
        title: "Key Press",
        subline: `${kind.mode} ${kind.key}`,
        key: kind.key,
        mode: kind.mode,
      });
      break;
    case "mouseButton":
      pushNode("mousePressNode", {
        title: "Mouse Press",
        subline: `${kind.mode} ${kind.button}`,
        button: kind.button,
        mode: kind.mode,
      });
      break;
    case "mouseMove":
      pushNode("mouseMoveNode", {
        title: "Move Mouse",
        subline: "Recorded position",
        x: kind.x,
        y: kind.y,
        coordinateMode: kind.coordinateMode,
      });
      break;
    case "scroll":
      pushNode("scrollMouseNode", {
        title: "Scroll Mouse",
        subline: "Recorded wheel",
        axis: kind.axis,
        amount: kind.amount,
      });
      break;
  }
}

function makeRecordedEdge(source: string, target: string): Edge {
  return {
    id: `e-recorded-${source}-${target}-${crypto.randomUUID()}`,
    source,
    sourceHandle: FLOW_OUT_HANDLE,
    target,
    targetHandle: FLOW_IN_HANDLE,
    type: "gradient",
    data: { kind: "trigger" },
  };
}
