import type { Connection, Edge } from "@xyflow/svelte";

export const FLOW_IN_HANDLE = "flow:in";
export const FLOW_OUT_HANDLE = "flow:out";
export const VALUE_HANDLE_PREFIX = "value:";
export const SUBMACRO_OUTPUT_HANDLE_PREFIX = "out:";

export type ConnectionKind = "trigger" | "value";

export function valueHandle(name: string) {
  return `${VALUE_HANDLE_PREFIX}${name}`;
}

export function submacroOutputHandle(outputId: string) {
  return `${SUBMACRO_OUTPUT_HANDLE_PREFIX}${outputId}`;
}

export function isValueHandle(handle?: string | null) {
  return Boolean(handle?.startsWith(VALUE_HANDLE_PREFIX));
}

export function valueHandleName(handle?: string | null) {
  if (!handle?.startsWith(VALUE_HANDLE_PREFIX)) return undefined;
  return handle.slice(VALUE_HANDLE_PREFIX.length);
}

export function isSubmacroOutputHandle(handle?: string | null) {
  return Boolean(handle?.startsWith(SUBMACRO_OUTPUT_HANDLE_PREFIX));
}

export function submacroOutputId(handle?: string | null) {
  if (!handle?.startsWith(SUBMACRO_OUTPUT_HANDLE_PREFIX)) return undefined;
  return handle.slice(SUBMACRO_OUTPUT_HANDLE_PREFIX.length);
}

export function connectionKindFromHandles(
  sourceHandle?: string | null,
  targetHandle?: string | null,
): ConnectionKind {
  return isValueHandle(sourceHandle) || isValueHandle(targetHandle) ? "value" : "trigger";
}

export function edgeKind(edge: Edge | Connection): ConnectionKind {
  const dataKind = "data" in edge ? edge.data?.kind : undefined;
  if (dataKind === "trigger" || dataKind === "value") return dataKind;
  return connectionKindFromHandles(edge.sourceHandle, edge.targetHandle);
}

export function isTriggerEdge(edge: Edge | Connection) {
  return edgeKind(edge) === "trigger";
}

export function isValueEdge(edge: Edge | Connection) {
  return edgeKind(edge) === "value";
}

export function normalizeTriggerHandle(handle?: string | null) {
  if (!handle || handle === FLOW_IN_HANDLE || handle === FLOW_OUT_HANDLE) {
    return FLOW_OUT_HANDLE;
  }

  return handle;
}

export function makeGraphEdge(connection: Connection): Edge {
  const kind = connectionKindFromHandles(connection.sourceHandle, connection.targetHandle);

  return {
    ...connection,
    id: `e-${connection.source}-${connection.sourceHandle ?? "out"}-${connection.target}-${connection.targetHandle ?? "in"}-${crypto.randomUUID()}`,
    type: "gradient",
    data: { kind },
    animated: kind === "value",
  };
}
