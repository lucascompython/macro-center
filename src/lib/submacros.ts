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

function cloneJson<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

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
  const cloned = cloneJson(node) as Node & {
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

  const selectedIds = new Set(selectedNodes.map((node) => node.id));
  const selected = nodes.filter(
    (node) => selectedIds.has(node.id) && node.type !== "subflowGroupNode",
  );
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

  const incomingTriggerEdges = incomingEdges.filter((edge) => edgeKind(edge) === "trigger");
  const incomingValueEdges = incomingEdges.filter((edge) => edgeKind(edge) === "value");
  const outgoingTriggerEdges = outgoingEdges.filter((edge) => edgeKind(edge) === "trigger");
  const outgoingValueEdges = outgoingEdges.filter((edge) => edgeKind(edge) === "value");

  const internalTriggerTargets = new Set(
    internalEdges.filter((edge) => edgeKind(edge) === "trigger").map((edge) => edge.target),
  );
  let entryNodeIds = [...new Set(incomingTriggerEdges.map((edge) => edge.target))];
  if (entryNodeIds.length === 0) {
    entryNodeIds = selected
      .filter((node) => !internalTriggerTargets.has(node.id))
      .map((node) => node.id);
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
      outputId = uniquePortId(portLabelFromHandle(sourceHandle, "done"), usedTriggerOutputIds);
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
      inputId = uniquePortId(valueHandleName(targetHandle) ?? "input", usedValueInputIds);
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
      outputId = uniquePortId(valueHandleName(sourceHandle) ?? "value", usedValueOutputIds);
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
    nodes: cloneJson(selected.map(detachNodeFromParent)),
    edges: cloneJson(internalEdges),
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
  const minX = Math.min(...selected.map((node) => node.position.x));
  const minY = Math.min(...selected.map((node) => node.position.y));
  const maxX = Math.max(...selected.map((node) => node.position.x + nodeWidth(node)));
  const maxY = Math.max(...selected.map((node) => node.position.y + nodeHeight(node)));
  const groupX = minX - padding;
  const groupY = minY - headerHeight - padding;
  const groupWidth = Math.max(420, maxX - minX + padding * 2);
  const groupHeight = Math.max(260, maxY - minY + padding * 2 + headerHeight);

  const subflowGroupId = crypto.randomUUID();
  const groupedNodes = selected.map((node) => ({
    ...cloneJson(node),
    position: {
      x: node.position.x - groupX,
      y: node.position.y - groupY,
    },
    parentId: subflowGroupId,
    extent: "parent" as const,
  }));

  submacro.nodes = cloneJson(groupedNodes.map(detachNodeFromParent));

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

  const rewrittenEdges = [
    ...untouchedEdges,
    ...internalEdges.map((edge) => ({
      ...edge,
      data: { ...(edge.data ?? {}), kind: edgeKind(edge) },
    })),
  ];

  for (const edge of incomingEdges) {
    if (edgeKind(edge) === "value") {
      const inputId = valueInputByTarget.get(
        `${edge.target}:${edge.targetHandle ?? valueHandle("value")}`,
      );
      if (!inputId) continue;
      rewrittenEdges.push({
        ...edge,
        id: makeEdgeId({ ...edge, target: submacroNodeId, targetHandle: valueHandle(inputId) }),
        target: submacroNodeId,
        targetHandle: valueHandle(inputId),
        data: { ...(edge.data ?? {}), kind: "value" },
        animated: true,
      });
      continue;
    }

    rewrittenEdges.push({
      ...edge,
      id: makeEdgeId({ ...edge, target: submacroNodeId, targetHandle: FLOW_IN_HANDLE }),
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
        id: makeEdgeId({ ...edge, source: submacroNodeId, sourceHandle: valueHandle(outputId) }),
        source: submacroNodeId,
        sourceHandle: valueHandle(outputId),
        data: { ...(edge.data ?? {}), kind: "value" },
        animated: true,
      });
      continue;
    }

    const sourceHandle = edge.sourceHandle ?? FLOW_OUT_HANDLE;
    const outputId = triggerOutputBySource.get(`${edge.source}:${sourceHandle}`) ?? triggerOutputs[0].id;
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
    nodes: [
      ...nodes.filter((node) => !selectedIds.has(node.id)),
      subflowGroupNode,
      ...groupedNodes,
      submacroNode,
    ],
    edges: rewrittenEdges,
    submacro,
  };
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

  return submacros.map((submacro) => {
    const group = groupsByDefinition.get(submacro.id);
    if (!group) return submacro;

    const childNodes = nodes.filter((node) => node.parentId === group.id);
    const childIds = new Set(childNodes.map((node) => node.id));
    const internalEdges = edges.filter(
      (edge) => childIds.has(edge.source) && childIds.has(edge.target),
    );

    const name = String(group.data?.title ?? submacro.name).trim() || "Subflow";

    return {
      ...submacro,
      name,
      nodes: cloneJson(childNodes.map(detachNodeFromParent)),
      edges: cloneJson(internalEdges),
      updatedAt: name === submacro.name ? submacro.updatedAt : new Date().toISOString(),
    };
  });
}
