import type { Edge, Node } from "@xyflow/svelte";
import {
  FLOW_IN_HANDLE,
  FLOW_OUT_HANDLE,
  edgeKind,
  submacroOutputHandle,
  valueHandle,
  valueHandleName,
} from "$lib/graph";
import type {
  SubmacroDefinition,
  SubmacroTriggerOutput,
  SubmacroValuePort,
} from "$lib/types";

interface CreateSubmacroOptions {
  nodes: Node[];
  edges: Edge[];
  selectedNodes: Node[];
  submacroCount: number;
}

interface CreateSubmacroResult {
  nodes: Node[];
  edges: Edge[];
  submacro: SubmacroDefinition;
}

type MeasuredNode = Node & {
  measured?: {
    width?: number;
    height?: number;
  };
  width?: number;
  height?: number;
};

function sanitizePortId(value: string) {
  return value
    .trim()
    .replace(/[^a-zA-Z0-9_]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .toLowerCase();
}

function uniquePortId(base: string, used: Set<string>) {
  const safeBase = sanitizePortId(base) || "port";
  let candidate = safeBase;
  let index = 2;
  while (used.has(candidate)) {
    candidate = `${safeBase}_${index}`;
    index += 1;
  }
  used.add(candidate);
  return candidate;
}

function portLabelFromHandle(handle?: string | null, fallback = "Done") {
  if (!handle || handle === FLOW_OUT_HANDLE) return fallback;
  return valueHandleName(handle) ?? handle.replace(/^out:/, "");
}

function makeEdgeId(edge: {
  source: string;
  target: string;
  sourceHandle?: string | null;
  targetHandle?: string | null;
}) {
  return `e-${edge.source}-${edge.sourceHandle ?? "out"}-${edge.target}-${edge.targetHandle ?? "in"}-${crypto.randomUUID()}`;
}

export function submacroNodeData(definition: SubmacroDefinition) {
  return {
    title: definition.name,
    subline: "Call visible subflow",
    definitionId: definition.id,
    triggerOutputs: definition.triggerOutputs,
    valueInputs: definition.valueInputs,
    valueOutputs: definition.valueOutputs,
  };
}

function subflowGroupData(definition: SubmacroDefinition) {
  return {
    title: definition.name,
    subline: "Visible subflow definition",
    kind: "subflow",
    definitionId: definition.id,
  };
}

function nodeWidth(node: Node) {
  const measured = node as MeasuredNode;
  return measured.width ?? measured.measured?.width ?? 320;
}

function nodeHeight(node: Node) {
  const measured = node as MeasuredNode;
  return measured.height ?? measured.measured?.height ?? 180;
}

function detachNodeFromParent(node: Node): Node {
  const cloned = structuredClone(node) as Node & {
    parentId?: string;
    extent?: string;
    expandParent?: boolean;
  };
  delete cloned.parentId;
  delete cloned.extent;
  delete cloned.expandParent;
  return cloned;
}

export function createSubmacroFromSelectionModel({
  nodes,
  edges,
  selectedNodes,
  submacroCount,
}: CreateSubmacroOptions): CreateSubmacroResult | undefined {
  if (selectedNodes.length === 0) return undefined;

  const selectedIds = new Set<string>();
  for (const node of selectedNodes) {
    selectedIds.add(node.id);
  }

  const selected: Node[] = [];
  for (const node of nodes) {
    if (selectedIds.has(node.id) && node.type !== "subflowGroupNode") {
      selected.push(node);
    }
  }
  if (selected.length === 0) return undefined;

  const internalEdges: Edge[] = [];
  const incomingEdges: Edge[] = [];
  const outgoingEdges: Edge[] = [];
  const untouchedEdges: Edge[] = [];

  for (const edge of edges) {
    const sourceSelected = selectedIds.has(edge.source);
    const targetSelected = selectedIds.has(edge.target);

    if (sourceSelected && targetSelected) {
      internalEdges.push(edge);
    } else if (!sourceSelected && targetSelected) {
      incomingEdges.push(edge);
    } else if (sourceSelected && !targetSelected) {
      outgoingEdges.push(edge);
    } else {
      untouchedEdges.push(edge);
    }
  }

  const incomingTriggerEdges: Edge[] = [];
  const incomingValueEdges: Edge[] = [];
  for (const edge of incomingEdges) {
    if (edgeKind(edge) === "trigger") {
      incomingTriggerEdges.push(edge);
    } else {
      incomingValueEdges.push(edge);
    }
  }

  const outgoingTriggerEdges: Edge[] = [];
  const outgoingValueEdges: Edge[] = [];
  for (const edge of outgoingEdges) {
    if (edgeKind(edge) === "trigger") {
      outgoingTriggerEdges.push(edge);
    } else {
      outgoingValueEdges.push(edge);
    }
  }

  const internalTriggerTargets = new Set<string>();
  for (const edge of internalEdges) {
    if (edgeKind(edge) === "trigger") {
      internalTriggerTargets.add(edge.target);
    }
  }

  const entryNodeIdSet = new Set<string>();
  for (const edge of incomingTriggerEdges) {
    entryNodeIdSet.add(edge.target);
  }
  let entryNodeIds = Array.from(entryNodeIdSet);
  if (entryNodeIds.length === 0) {
    entryNodeIds = [];
    for (const node of selected) {
      if (!internalTriggerTargets.has(node.id)) {
        entryNodeIds.push(node.id);
      }
    }
  }
  if (entryNodeIds.length === 0) {
    entryNodeIds = [selected[0].id];
  }

  const usedTriggerOutputIds = new Set<string>();
  const triggerOutputs: SubmacroTriggerOutput[] = [];
  const triggerOutputBySource = new Map<string, string>();
  const boundaryOutputs: SubmacroDefinition["boundaryOutputs"] = [];

  for (const edge of outgoingTriggerEdges) {
    const sourceHandle = edge.sourceHandle ?? FLOW_OUT_HANDLE;
    const mapKey = `${edge.source}:${sourceHandle}`;
    let outputId = triggerOutputBySource.get(mapKey);
    if (!outputId) {
      outputId = uniquePortId(
        portLabelFromHandle(sourceHandle, "done"),
        usedTriggerOutputIds,
      );
      triggerOutputBySource.set(mapKey, outputId);
      triggerOutputs.push({
        id: outputId,
        label: portLabelFromHandle(sourceHandle, "Done"),
      });
      boundaryOutputs.push({
        outputId,
        sourceNodeId: edge.source,
        sourceHandle,
      });
    }
  }

  if (triggerOutputs.length === 0) {
    triggerOutputs.push({ id: "done", label: "Done" });
  }

  const usedValueInputIds = new Set<string>();
  const valueInputs: SubmacroValuePort[] = [];
  const valueInputByTarget = new Map<string, string>();
  const boundaryValueInputs: SubmacroDefinition["boundaryValueInputs"] = [];

  for (const edge of incomingValueEdges) {
    const targetHandle = edge.targetHandle ?? valueHandle("value");
    const mapKey = `${edge.target}:${targetHandle}`;
    let inputId = valueInputByTarget.get(mapKey);
    if (!inputId) {
      inputId = uniquePortId(
        valueHandleName(targetHandle) ?? "input",
        usedValueInputIds,
      );
      valueInputByTarget.set(mapKey, inputId);
      valueInputs.push({
        id: inputId,
        label: valueHandleName(targetHandle) ?? "Input",
        type: "text",
        defaultValue: "",
      });
      boundaryValueInputs.push({
        inputId,
        targetNodeId: edge.target,
        targetHandle,
      });
    }
  }

  const usedValueOutputIds = new Set<string>();
  const valueOutputs: SubmacroValuePort[] = [];
  const valueOutputBySource = new Map<string, string>();
  const valueOutputBindings: SubmacroDefinition["valueOutputBindings"] = [];

  for (const edge of outgoingValueEdges) {
    const sourceHandle = edge.sourceHandle ?? valueHandle("value");
    const mapKey = `${edge.source}:${sourceHandle}`;
    let outputId = valueOutputBySource.get(mapKey);
    if (!outputId) {
      outputId = uniquePortId(
        valueHandleName(sourceHandle) ?? "value",
        usedValueOutputIds,
      );
      valueOutputBySource.set(mapKey, outputId);
      valueOutputs.push({
        id: outputId,
        label: valueHandleName(sourceHandle) ?? "Value",
        type: "text",
      });
      valueOutputBindings.push({
        outputId,
        sourceNodeId: edge.source,
        sourceHandle,
      });
    }
  }

  const now = new Date().toISOString();
  const submacro: SubmacroDefinition = {
    id: crypto.randomUUID(),
    name: `Submacro ${submacroCount + 1}`,
    nodes: [],
    edges: structuredClone(internalEdges),
    variables: [],
    entryNodeIds,
    triggerOutputs,
    valueInputs,
    valueOutputs,
    boundaryValueInputs,
    valueOutputBindings,
    boundaryOutputs,
    createdAt: now,
    updatedAt: now,
  };

  const padding = 48;
  const headerHeight = 48;
  let minX = Number.POSITIVE_INFINITY;
  let minY = Number.POSITIVE_INFINITY;
  let maxX = Number.NEGATIVE_INFINITY;
  let maxY = Number.NEGATIVE_INFINITY;
  for (const node of selected) {
    minX = Math.min(minX, node.position.x);
    minY = Math.min(minY, node.position.y);
    maxX = Math.max(maxX, node.position.x + nodeWidth(node));
    maxY = Math.max(maxY, node.position.y + nodeHeight(node));
  }
  const groupX = minX - padding;
  const groupY = minY - headerHeight - padding;
  const groupWidth = Math.max(420, maxX - minX + padding * 2);
  const groupHeight = Math.max(260, maxY - minY + padding * 2 + headerHeight);

  const subflowGroupId = crypto.randomUUID();
  const groupedNodes = new Array<Node>(selected.length);
  for (let index = 0; index < selected.length; index += 1) {
    const node = selected[index];
    groupedNodes[index] = {
      ...structuredClone(node),
      position: {
        x: node.position.x - groupX,
        y: node.position.y - groupY,
      },
      parentId: subflowGroupId,
      extent: "parent" as const,
    };
  }

  submacro.nodes = detachNodesFromParents(groupedNodes);

  const subflowGroupNode: Node = {
    id: subflowGroupId,
    type: "subflowGroupNode",
    position: { x: groupX, y: groupY },
    data: subflowGroupData(submacro),
    style: `width: ${groupWidth}px; height: ${groupHeight}px;`,
    selectable: true,
  };

  const submacroNodeId = crypto.randomUUID();
  const submacroNode: Node = {
    id: submacroNodeId,
    type: "submacroNode",
    position: { x: groupX + groupWidth + 80, y: groupY + 64 },
    data: submacroNodeData(submacro),
  };

  const rewrittenEdges: Edge[] = [];
  for (const edge of untouchedEdges) {
    rewrittenEdges.push(edge);
  }
  for (const edge of internalEdges) {
    rewrittenEdges.push({
      ...edge,
      data: { ...(edge.data ?? {}), kind: edgeKind(edge) },
    });
  }

  for (const edge of incomingEdges) {
    if (edgeKind(edge) === "value") {
      const inputId = valueInputByTarget.get(
        `${edge.target}:${edge.targetHandle ?? valueHandle("value")}`,
      );
      if (!inputId) continue;
      rewrittenEdges.push({
        ...edge,
        id: makeEdgeId({
          ...edge,
          target: submacroNodeId,
          targetHandle: valueHandle(inputId),
        }),
        target: submacroNodeId,
        targetHandle: valueHandle(inputId),
        data: { ...(edge.data ?? {}), kind: "value" },
        animated: true,
      });
      continue;
    }

    rewrittenEdges.push({
      ...edge,
      id: makeEdgeId({
        ...edge,
        target: submacroNodeId,
        targetHandle: FLOW_IN_HANDLE,
      }),
      target: submacroNodeId,
      targetHandle: FLOW_IN_HANDLE,
      data: { ...(edge.data ?? {}), kind: "trigger" },
    });
  }

  for (const edge of outgoingEdges) {
    if (edgeKind(edge) === "value") {
      const outputId = valueOutputBySource.get(
        `${edge.source}:${edge.sourceHandle ?? valueHandle("value")}`,
      );
      if (!outputId) continue;
      rewrittenEdges.push({
        ...edge,
        id: makeEdgeId({
          ...edge,
          source: submacroNodeId,
          sourceHandle: valueHandle(outputId),
        }),
        source: submacroNodeId,
        sourceHandle: valueHandle(outputId),
        data: { ...(edge.data ?? {}), kind: "value" },
        animated: true,
      });
      continue;
    }

    const sourceHandle = edge.sourceHandle ?? FLOW_OUT_HANDLE;
    const outputId =
      triggerOutputBySource.get(`${edge.source}:${sourceHandle}`) ??
      triggerOutputs[0].id;
    rewrittenEdges.push({
      ...edge,
      id: makeEdgeId({
        ...edge,
        source: submacroNodeId,
        sourceHandle: submacroOutputHandle(outputId),
      }),
      source: submacroNodeId,
      sourceHandle: submacroOutputHandle(outputId),
      data: { ...(edge.data ?? {}), kind: "trigger" },
    });
  }

  return {
    nodes: createSubflowNodeList(
      nodes,
      selectedIds,
      subflowGroupNode,
      groupedNodes,
      submacroNode,
    ),
    edges: rewrittenEdges,
    submacro,
  };
}

function createSubflowNodeList(
  nodes: Node[],
  selectedIds: Set<string>,
  subflowGroupNode: Node,
  groupedNodes: Node[],
  submacroNode: Node,
) {
  const nextNodes: Node[] = [];
  for (const node of nodes) {
    if (!selectedIds.has(node.id)) nextNodes.push(node);
  }
  nextNodes.push(subflowGroupNode);
  for (const node of groupedNodes) {
    nextNodes.push(node);
  }
  nextNodes.push(submacroNode);
  return nextNodes;
}

export function syncSubmacroDefinitionsFromSubflows(
  nodes: Node[],
  edges: Edge[],
  submacros: SubmacroDefinition[],
) {
  const groupsByDefinition = new Map<string, Node>();
  for (const node of nodes) {
    const definitionId = node.data?.definitionId;
    if (node.type === "subflowGroupNode" && typeof definitionId === "string") {
      groupsByDefinition.set(definitionId, node);
    }
  }

  if (groupsByDefinition.size === 0) return submacros;

  const childrenByGroup = new Map<string, Node[]>();
  for (const node of nodes) {
    if (!node.parentId) continue;
    const group = childrenByGroup.get(node.parentId);
    if (group) {
      group.push(node);
    } else {
      childrenByGroup.set(node.parentId, [node]);
    }
  }

  const syncedSubmacros = new Array<SubmacroDefinition>(submacros.length);
  for (let index = 0; index < submacros.length; index += 1) {
    const submacro = submacros[index];
    const group = groupsByDefinition.get(submacro.id);
    if (!group) {
      syncedSubmacros[index] = submacro;
      continue;
    }

    const childNodes = childrenByGroup.get(group.id) ?? [];
    const childIds = new Set<string>();
    for (const node of childNodes) {
      childIds.add(node.id);
    }

    const internalEdges: Edge[] = [];
    for (const edge of edges) {
      if (childIds.has(edge.source) && childIds.has(edge.target)) {
        internalEdges.push(edge);
      }
    }

    const name = String(group.data?.title ?? submacro.name).trim() || "Subflow";

    syncedSubmacros[index] = {
      ...submacro,
      name,
      nodes: detachNodesFromParents(childNodes),
      edges: structuredClone(internalEdges),
      updatedAt:
        name === submacro.name ? submacro.updatedAt : new Date().toISOString(),
    };
  }

  return syncedSubmacros;
}

function detachNodesFromParents(nodes: Node[]) {
  const detached = new Array<Node>(nodes.length);
  for (let index = 0; index < nodes.length; index += 1) {
    detached[index] = detachNodeFromParent(nodes[index]);
  }
  return detached;
}
