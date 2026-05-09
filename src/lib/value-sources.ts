import type { Edge, Node } from "@xyflow/svelte";
import { valueHandle, valueHandleName } from "$lib/graph";
import { valueToInput } from "$lib/logic";
import type { MacroValue, VariableDefinition } from "$lib/types";

interface HandleConnectionLike {
  source: string;
  sourceHandle?: string | null;
}

interface InternalNodeLike {
  internals?: {
    userNode?: Node;
  };
}

interface FlowStoreLike {
  edges: Edge[];
  connectionLookup: Map<string, Map<string, HandleConnectionLike>>;
  nodeLookup: Map<string, InternalNodeLike>;
}

interface ValueSourceLookup {
  variables: VariableDefinition[];
  snapshot: Record<string, MacroValue>;
}

export interface ConnectedValuePreview {
  text: string;
  title: string;
  sourceNodeId: string;
}

function connectionLookupKey(nodeId: string, type: "source" | "target", handleId?: string | null) {
  return `${nodeId}-${type}${handleId ? `-${handleId}` : ""}`;
}

function hasSnapshotValue(snapshot: Record<string, MacroValue>, name: string) {
  return Object.prototype.hasOwnProperty.call(snapshot, name);
}

function variablePreview(name: string, lookup: ValueSourceLookup) {
  if (!name) return "";
  if (hasSnapshotValue(lookup.snapshot, name)) {
    return valueToInput(lookup.snapshot[name]);
  }

  const variable = lookup.variables.find((definition) => definition.name === name);
  if (variable) return valueToInput(variable.defaultValue);

  return `$${name}`;
}

function nodeTitle(node: Node) {
  const data = node.data as { title?: unknown } | undefined;
  return String(data?.title ?? node.type ?? node.id);
}

function previewValueFromNode(node: Node, outputName: string, lookup: ValueSourceLookup) {
  const data = (node.data ?? {}) as Record<string, unknown>;

  switch (node.type) {
    case "valueNode":
      return valueToInput(data.value as MacroValue);
    case "getVariableNode":
      return variablePreview(String(data.variableName ?? ""), lookup);
    case "setVariableNode":
    case "updateVariableNode":
      return valueToInput(data.value as MacroValue);
    case "compareNode":
      return "true/false";
    case "submacroNode":
      return `${nodeTitle(node)}.${outputName}`;
    default: {
      const directValue = data[outputName];
      if (directValue !== undefined) return valueToInput(directValue as MacroValue);
      return `${nodeTitle(node)}.${outputName}`;
    }
  }
}

export function resolveConnectedValuePreview(
  store: FlowStoreLike,
  nodeId: string,
  inputName: string,
  lookup: ValueSourceLookup,
): ConnectedValuePreview | undefined {
  // connectionLookup is mutable in xyflow; touching edges makes Svelte recalculate on edge edits.
  void store.edges;

  const targetHandle = valueHandle(inputName);
  const connections = store.connectionLookup.get(connectionLookupKey(nodeId, "target", targetHandle));
  const connection = connections?.values().next().value;
  if (!connection) return undefined;

  const sourceNode = store.nodeLookup.get(connection.source)?.internals?.userNode;
  if (!sourceNode) return undefined;

  const outputName = valueHandleName(connection.sourceHandle) ?? "value";
  const text = previewValueFromNode(sourceNode, outputName, lookup);

  return {
    text,
    title: `Connected to ${nodeTitle(sourceNode)}`,
    sourceNodeId: sourceNode.id,
  };
}
