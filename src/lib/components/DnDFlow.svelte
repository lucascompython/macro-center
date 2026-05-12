<script lang="ts">
  import {
    SvelteFlow,
    Background,
    BackgroundVariant,
    MiniMap,
    Controls,
    Panel,
    useSvelteFlow,
    type Node,
    type Edge,
    type NodeTypes,
    type EdgeTypes,
    type IsValidConnection,
    type Connection,
    type OnBeforeConnect,
  } from "@xyflow/svelte";


  import "./nodes/node-styles.css";

  import KeyBindNode from "./nodes/KeyBindNode.svelte";
  import TypeNode from "./nodes/TypeNode.svelte";
  import KeyNode from "./nodes/KeyNode.svelte";
  import MousePressNode from "./nodes/MousePressNode.svelte";
  import MouseMoveNode from "./nodes/MouseMoveNode.svelte";
  import DelayNode from "./nodes/DelayNode.svelte";
  import ScrollMouseNode from "./nodes/ScrollMouseNode.svelte";
  import ConditionalNode from "./nodes/ConditionalNode.svelte";
  import SetVariableNode from "./nodes/SetVariableNode.svelte";
  import GetVariableNode from "./nodes/GetVariableNode.svelte";
  import UpdateVariableNode from "./nodes/UpdateVariableNode.svelte";
  import CompareNode from "./nodes/CompareNode.svelte";
  import ValueNode from "./nodes/ValueNode.svelte";
  import RepeatLoopNode from "./nodes/RepeatLoopNode.svelte";
  import ForEachLoopNode from "./nodes/ForEachLoopNode.svelte";
  import WhileLoopNode from "./nodes/WhileLoopNode.svelte";
  import BreakLoopNode from "./nodes/BreakLoopNode.svelte";
  import ContinueLoopNode from "./nodes/ContinueLoopNode.svelte";
  import SubmacroNode from "./nodes/SubmacroNode.svelte";
  import SubflowGroupNode from "./nodes/SubflowGroupNode.svelte";
  import GradientEdge from "./nodes/GradientEdge.svelte";
  import VariablesPanel from "$lib/components/VariablesPanel.svelte";

  import { initialNodes, initialEdges } from "$lib/initial-nodes";
  import {
    edgeKind,
    isValueHandle,
    makeGraphEdge,
  } from "$lib/graph";
  import {
    createSubmacroFromSelectionModel,
    submacroNodeData,
    syncSubmacroDefinitionsFromSubflows,
  } from "$lib/submacros";
  import type {
    MacroProjectData,
    MacroValue,
    SubmacroDefinition,
    VariableDefinition,
  } from "$lib/types";
  import {
    SUBFLOW_RENAME_CONTEXT,
    VALUE_SOURCE_CONTEXT,
    type RenameSubflow,
    type ValueSourceContext,
  } from "$lib/editor-context";

  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";
  import {
    writeTextFile,
    readTextFile,
  } from "@tauri-apps/plugin-fs";
  import {
    recordedMacroToGraph,
    type RecordedMacro,
    type RecordingMouseMode,
  } from "$lib/recording";
  import { MacroRunner } from "$lib/runner/MacroRunner";
  import {
    MODIFIER_KEYS,
    modifierShortcutPreview,
    shortcutFromKeyboardEvent,
  } from "$lib/shortcuts";
  import { onDestroy, onMount, setContext } from "svelte";

  type EditorSnapshot = {
    nodes: Node[];
    edges: Edge[];
    variables: VariableDefinition[];
    submacros: SubmacroDefinition[];
  };

  type GraphClipboard = {
    nodes: Node[];
    edges: Edge[];
  };

  type ContextMenuState = {
    x: number;
    y: number;
    flowPosition: { x: number; y: number };
    target: "pane" | "node" | "selection";
    delayBaselines: Record<string, number>;
    delayMultiplier: number;
  };

  // use $state.raw for performance as recommended by xyflow docs
  let nodes = $state.raw(initialNodes);
  let edges = $state.raw(initialEdges);
  let variables = $state.raw([] as VariableDefinition[]);
  let submacros = $state.raw([] as SubmacroDefinition[]);
  let selectedNodes = $state.raw([] as typeof nodes);
  let variableSnapshot = $state({} as Record<string, MacroValue>);
  let showVariables = $state(false);
  let undoStack = $state.raw([] as EditorSnapshot[]);
  let redoStack = $state.raw([] as EditorSnapshot[]);
  let copiedGraph = $state.raw<GraphClipboard | undefined>();
  let contextMenu = $state<ContextMenuState | undefined>();
  let pasteIndex = 0;
  let applyingHistory = false;
  let historyTimer: ReturnType<typeof setTimeout> | undefined;

  const valueSourceContext: ValueSourceContext = {
    getVariables: () => variables,
    getSnapshot: () => variableSnapshot,
  };

  const renameSubflowDefinition: RenameSubflow = (definitionId, rawName) => {
    const trimmedName = rawName.trim();
    const name = trimmedName || "Subflow";
    const now = new Date().toISOString();

    submacros = submacros.map((submacro) =>
      submacro.id === definitionId
        ? {
            ...submacro,
            name,
            updatedAt: now,
          }
        : submacro,
    );

    nodes = nodes.map((node) => {
      const data = node.data as { definitionId?: string } | undefined;
      if (
        (node.type === "subflowGroupNode" || node.type === "submacroNode") &&
        data?.definitionId === definitionId
      ) {
        return {
          ...node,
          data: {
            ...(node.data ?? {}),
            title: rawName || name,
          },
        };
      }

      return node;
    });
  };

  setContext(SUBFLOW_RENAME_CONTEXT, renameSubflowDefinition);
  setContext(VALUE_SOURCE_CONTEXT, valueSourceContext);

  function cloneData<T>(value: T): T {
    return structuredClone(value);
  }

  function currentEditorSnapshot(): EditorSnapshot {
    return {
      nodes: cloneData(nodes),
      edges: cloneData(edges),
      variables: cloneData(variables),
      submacros: cloneData(submacros),
    };
  }

  function snapshotSignature(snapshot: EditorSnapshot) {
    return JSON.stringify(snapshot);
  }

  let lastHistorySnapshot = currentEditorSnapshot();
  let lastHistorySignature = snapshotSignature(lastHistorySnapshot);

  function commitHistory() {
    if (applyingHistory) return;

    const snapshot = currentEditorSnapshot();
    const signature = snapshotSignature(snapshot);
    if (signature === lastHistorySignature) return;

    undoStack = [...undoStack.slice(-79), lastHistorySnapshot];
    redoStack = [];
    lastHistorySnapshot = snapshot;
    lastHistorySignature = signature;
  }

  function scheduleHistoryCommit() {
    if (applyingHistory) return;
    if (historyTimer) clearTimeout(historyTimer);
    historyTimer = setTimeout(commitHistory, 220);
  }

  function flushHistoryCommit() {
    if (historyTimer) {
      clearTimeout(historyTimer);
      historyTimer = undefined;
    }
    commitHistory();
  }

  function restoreSnapshot(snapshot: EditorSnapshot) {
    applyingHistory = true;
    nodes = cloneData(snapshot.nodes);
    edges = cloneData(snapshot.edges);
    variables = cloneData(snapshot.variables);
    submacros = cloneData(snapshot.submacros);
    selectedNodes = [];
    lastHistorySnapshot = cloneData(snapshot);
    lastHistorySignature = snapshotSignature(snapshot);
    queueMicrotask(() => {
      applyingHistory = false;
    });
  }

  function undo() {
    flushHistoryCommit();
    const snapshot = undoStack.at(-1);
    if (!snapshot) return;

    const current = currentEditorSnapshot();
    undoStack = undoStack.slice(0, -1);
    redoStack = [current, ...redoStack.slice(0, 79)];
    restoreSnapshot(snapshot);
    closeContextMenu();
  }

  function redo() {
    flushHistoryCommit();
    const snapshot = redoStack[0];
    if (!snapshot) return;

    const current = currentEditorSnapshot();
    redoStack = redoStack.slice(1);
    undoStack = [...undoStack.slice(-79), current];
    restoreSnapshot(snapshot);
    closeContextMenu();
  }

  $effect(() => {
    nodes;
    edges;
    variables;
    submacros;
    scheduleHistoryCommit();
  });

  const nodeTypes: NodeTypes = {
    keyBindNode: KeyBindNode,
    typeNode: TypeNode,
    keyNode: KeyNode,
    mousePressNode: MousePressNode,
    mouseMoveNode: MouseMoveNode,
    delayNode: DelayNode,
    scrollMouseNode: ScrollMouseNode,
    conditionalNode: ConditionalNode,
    setVariableNode: SetVariableNode,
    getVariableNode: GetVariableNode,
    updateVariableNode: UpdateVariableNode,
    compareNode: CompareNode,
    valueNode: ValueNode,
    repeatLoopNode: RepeatLoopNode,
    forEachLoopNode: ForEachLoopNode,
    whileLoopNode: WhileLoopNode,
    breakLoopNode: BreakLoopNode,
    continueLoopNode: ContinueLoopNode,
    submacroNode: SubmacroNode,
    subflowGroupNode: SubflowGroupNode,
  };

  const edgeTypes: EdgeTypes = {
    gradient: GradientEdge,
  };

  const defaultEdgeOptions = {
    type: "gradient",
  };

  // validate connections - prevent connecting to self
  const isValidConnection: IsValidConnection = (connection) => {
    if (connection.source === connection.target) {
      return false;
    }

    const sourceIsValue = isValueHandle(connection.sourceHandle);
    const targetIsValue = isValueHandle(connection.targetHandle);
    return sourceIsValue === targetIsValue;
  };

  const onBeforeConnect: OnBeforeConnect = (connection: Connection) => {
    if (!isValidConnection(connection)) return false;
    return makeGraphEdge(connection);
  };

  // DnD Hook
  const { screenToFlowPosition, toObject, setViewport } = useSvelteFlow();

  let runner: MacroRunner | undefined = $state();
  let isRunning = $state(false);
  let isRecording = $state(false);
  let recordingMode = $state<RecordingMouseMode>("movesBeforeClicks");
  let recordingError = $state("");
  let pendingRecordedMacro = $state<RecordedMacro | undefined>();
  let recordedShortcut = $state("");
  let recordingShortcut = $state(false);

  function toggleExecution() {
    if (isRunning) {
      runner?.cleanup();
      isRunning = false;
    } else {
      const syncedSubmacros = syncSubmacroDefinitionsFromSubflows(nodes, edges, submacros);
      submacros = syncedSubmacros;

      if (!runner) {
        runner = new MacroRunner(nodes, edges, variables, syncedSubmacros, (snapshot) => {
          variableSnapshot = { ...snapshot };
        });
      }
      runner.updateGraph(nodes, edges, variables, syncedSubmacros);
      isRunning = true;
    }
  }

  async function toggleRecording() {
    if (isRecording) {
      await stopRecording();
    } else {
      await startRecording();
    }
  }

  async function startRecording() {
    recordingError = "";
    pendingRecordedMacro = undefined;
    try {
      await invoke("start_macro_recording", { mode: recordingMode });
      isRecording = true;
    } catch (error) {
      recordingError = String(error);
      console.error("Failed to start recording:", error);
    }
  }

  async function stopRecording() {
    recordingError = "";
    try {
      const recorded = await invoke<RecordedMacro>("stop_macro_recording");
      isRecording = false;

      if (recorded.actions.length === 0) {
        recordingError = "No actions captured.";
        return;
      }

      pendingRecordedMacro = recorded;
      recordedShortcut = "";
      recordingShortcut = true;
    } catch (error) {
      recordingError = String(error);
      console.error("Failed to stop recording:", error);
    }
  }

  function startShortcutRecording() {
    recordingShortcut = true;
    recordedShortcut = "...";
  }

  function shortcutReady(shortcut = recordedShortcut) {
    return Boolean(shortcut) && shortcut !== "..." && !shortcut.endsWith("+...");
  }

  function handleRecordedShortcutKeyDown(event: KeyboardEvent) {
    if (!recordingShortcut) return;

    event.preventDefault();
    event.stopPropagation();

    if (MODIFIER_KEYS.has(event.key)) {
      recordedShortcut = modifierShortcutPreview(event);
      return;
    }

    const shortcut = shortcutFromKeyboardEvent(event);
    if (shortcut) {
      recordedShortcut = shortcut;
      recordingShortcut = false;
    }
  }

  function createRecordedMacro(shortcut = recordedShortcut.trim()) {
    if (!pendingRecordedMacro) return;

    const graph = recordedMacroToGraph(
      pendingRecordedMacro,
      shortcutReady(shortcut) ? shortcut : "",
      screenToFlowPosition({ x: 320, y: 120 }),
    );

    const selectedRecordedNodes = graph.nodes.map((node) => ({
      ...node,
      selected: true,
    }));

    nodes = [
      ...nodes.map((node) => ({ ...node, selected: false })),
      ...selectedRecordedNodes,
    ];
    edges = [...edges, ...graph.edges];
    selectedNodes = selectedRecordedNodes;
    pendingRecordedMacro = undefined;
    recordedShortcut = "";
    recordingShortcut = false;
    closeContextMenu();
  }

  function discardRecordedMacro() {
    pendingRecordedMacro = undefined;
    recordedShortcut = "";
    recordingShortcut = false;
  }

  onDestroy(() => {
    if (historyTimer) {
      clearTimeout(historyTimer);
    }

    if (runner) {
      runner.cleanup();
    }

    if (isRecording) {
      void invoke("stop_macro_recording");
    }
  });

  function closeContextMenu() {
    contextMenu = undefined;
  }

  function isEditingTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return Boolean(
      target.closest("input, textarea, select, [contenteditable='true']"),
    );
  }

  function selectedNodeIdsWithDescendants() {
    const copiedIds = new Set(selectedNodes.map((node) => node.id));
    let changed = true;

    while (changed) {
      changed = false;
      for (const node of nodes) {
        if (node.parentId && copiedIds.has(node.parentId) && !copiedIds.has(node.id)) {
          copiedIds.add(node.id);
          changed = true;
        }
      }
    }

    return copiedIds;
  }

  function selectedDelayBaselines() {
    const selectedIds = selectedNodeIdsWithDescendants();
    const baselines: Record<string, number> = {};

    for (const node of nodes) {
      if (selectedIds.has(node.id) && node.type === "delayNode") {
        baselines[node.id] = Math.max(0, Number(node.data.delay ?? 1000) || 0);
      }
    }

    return baselines;
  }

  function selectedDelayCount() {
    return Object.keys(contextMenu?.delayBaselines ?? {}).length;
  }

  function deleteSelectedDelayNodes() {
    const delayIds = new Set(Object.keys(contextMenu?.delayBaselines ?? {}));
    if (delayIds.size === 0) return;

    const triggerEdges = edges.filter((edge) => edgeKind(edge) === "trigger");
    const outgoingBySource = new Map<string, Edge[]>();
    const keptEdges = edges.filter((edge) => !delayIds.has(edge.source) && !delayIds.has(edge.target));
    const edgeKeys = new Set(
      keptEdges.map(
        (edge) =>
          `${edge.source}:${edge.sourceHandle ?? ""}->${edge.target}:${edge.targetHandle ?? ""}`,
      ),
    );
    const bypassEdges: Edge[] = [];

    for (const edge of triggerEdges) {
      const sourceEdges = outgoingBySource.get(edge.source) ?? [];
      sourceEdges.push(edge);
      outgoingBySource.set(edge.source, sourceEdges);
    }

    function reachableOutputs(nodeId: string, visited = new Set<string>()): Edge[] {
      if (visited.has(nodeId)) return [];
      visited.add(nodeId);

      const outgoing = outgoingBySource.get(nodeId) ?? [];
      const outputs: Edge[] = [];

      for (const edge of outgoing) {
        if (delayIds.has(edge.target)) {
          outputs.push(...reachableOutputs(edge.target, visited));
        } else {
          outputs.push(edge);
        }
      }

      return outputs;
    }

    for (const incomingEdge of triggerEdges) {
      if (!delayIds.has(incomingEdge.target) || delayIds.has(incomingEdge.source)) continue;

      for (const outgoingEdge of reachableOutputs(incomingEdge.target)) {
        if (outgoingEdge.target === incomingEdge.source) continue;

        const edgeKey = `${incomingEdge.source}:${incomingEdge.sourceHandle ?? ""}->${outgoingEdge.target}:${outgoingEdge.targetHandle ?? ""}`;
        if (edgeKeys.has(edgeKey)) continue;
        edgeKeys.add(edgeKey);

        bypassEdges.push(
          makeGraphEdge({
            source: incomingEdge.source,
            sourceHandle: incomingEdge.sourceHandle ?? null,
            target: outgoingEdge.target,
            targetHandle: outgoingEdge.targetHandle ?? null,
          }),
        );
      }
    }

    nodes = nodes.filter((node) => !delayIds.has(node.id));
    edges = [...keptEdges, ...bypassEdges];
    selectedNodes = selectedNodes.filter((node) => !delayIds.has(node.id));
    closeContextMenu();
  }

  function applyDelayMultiplier(rawMultiplier: number) {
    if (!contextMenu) return;

    const multiplier = Number(rawMultiplier) || 0;
    const baselines = contextMenu.delayBaselines;
    contextMenu = {
      ...contextMenu,
      delayMultiplier: multiplier,
    };

    nodes = nodes.map((node) => {
      const baseline = baselines[node.id];
      if (baseline === undefined || node.type !== "delayNode") return node;

      return {
        ...node,
        data: {
          ...(node.data ?? {}),
          delay: Math.round(baseline * multiplier),
        },
      };
    });
  }

  function optimizeTypingTextSelection() {
    const runs = findTypingOptimizationRuns();
    if (runs.length === 0) return;

    const consumedIds = new Set(runs.flatMap((run) => [...run.consumedIds]));
    const replacementNodes = runs.map((run) => {
      const firstNode = run.nodes[0];
      const replacement: Node = {
        id: crypto.randomUUID(),
        type: "typeNode",
        position: firstNode.position,
        data: {
          title: "Type Text",
          subline: "Optimized typed text",
          text: run.text,
        },
        selected: true,
      };

      if (firstNode.parentId) {
        replacement.parentId = firstNode.parentId;
        replacement.extent = firstNode.extent;
      }
      if (firstNode.origin) {
        replacement.origin = firstNode.origin;
      }

      return replacement;
    });
    const replacementByRun = new Map(runs.map((run, index) => [run, replacementNodes[index]]));
    const keptEdges = edges.filter(
      (edge) => !consumedIds.has(edge.source) && !consumedIds.has(edge.target),
    );
    const edgeKeys = new Set(
      keptEdges.map(
        (edge) =>
          `${edge.source}:${edge.sourceHandle ?? ""}->${edge.target}:${edge.targetHandle ?? ""}`,
      ),
    );
    const replacementEdges: Edge[] = [];

    for (const run of runs) {
      const replacement = replacementByRun.get(run);
      if (!replacement) continue;

      for (const incomingEdge of run.incomingEdges) {
        addReplacementEdge(
          replacementEdges,
          edgeKeys,
          incomingEdge.source,
          incomingEdge.sourceHandle ?? null,
          replacement.id,
          incomingEdge.targetHandle ?? null,
        );
      }

      for (const outgoingEdge of run.outgoingEdges) {
        addReplacementEdge(
          replacementEdges,
          edgeKeys,
          replacement.id,
          outgoingEdge.sourceHandle ?? null,
          outgoingEdge.target,
          outgoingEdge.targetHandle ?? null,
        );
      }
    }

    nodes = [
      ...nodes.filter((node) => !consumedIds.has(node.id)).map((node) => ({ ...node, selected: false })),
      ...replacementNodes,
    ];
    edges = [...keptEdges, ...replacementEdges];
    selectedNodes = replacementNodes;
    closeContextMenu();
  }

  function canOptimizeTypingTextSelection() {
    return findTypingOptimizationRuns().length > 0;
  }

  type TypingRun = {
    text: string;
    nodes: Node[];
    consumedIds: Set<string>;
    incomingEdges: Edge[];
    outgoingEdges: Edge[];
  };

  function findTypingOptimizationRuns(): TypingRun[] {
    const selectedIds = selectedNodeIdsWithDescendants();
    if (selectedIds.size === 0) return [];

    const nodesById = new Map(nodes.map((node) => [node.id, node]));
    const triggerEdges = edges.filter((edge) => edgeKind(edge) === "trigger");
    const outgoingBySource = groupEdges(triggerEdges, "source");
    const incomingByTarget = groupEdges(triggerEdges, "target");
    const consumed = new Set<string>();
    const runs: TypingRun[] = [];

    for (const node of nodes) {
      if (!selectedIds.has(node.id) || consumed.has(node.id)) {
        continue;
      }
      if (node.type !== "keyNode") continue;

      const run = parseTypingRun(
        node.id,
        selectedIds,
        consumed,
        nodesById,
        outgoingBySource,
        incomingByTarget,
      );
      if (!run) continue;

      for (const id of run.consumedIds) {
        consumed.add(id);
      }
      runs.push(run);
    }

    return runs;
  }

  function parseTypingRun(
    startNodeId: string,
    selectedIds: Set<string>,
    alreadyConsumed: Set<string>,
    nodesById: Map<string, Node>,
    outgoingBySource: Map<string, Edge[]>,
    incomingByTarget: Map<string, Edge[]>,
  ): TypingRun | undefined {
    const consumedIds = new Set<string>();
    const textParts: string[] = [];
    let currentNodeId: string | undefined = startNodeId;
    let lastNodeId = startNodeId;

    while (currentNodeId) {
      if (!selectedIds.has(currentNodeId) || alreadyConsumed.has(currentNodeId) || consumedIds.has(currentNodeId)) {
        break;
      }

      const node = nodesById.get(currentNodeId);
      if (node?.type !== "keyNode") break;

      const text = textForKeyNode(node);
      if (text === undefined) break;

      const mode = String(node.data.mode ?? "click").toLowerCase();
      if (mode === "press" || mode === "click") {
        textParts.push(text);
      } else if (mode !== "release" || textParts.length === 0) {
        break;
      }

      consumedIds.add(node.id);
      lastNodeId = node.id;

      const next = nextPrintableKeyNode(node.id, selectedIds, alreadyConsumed, consumedIds, nodesById, outgoingBySource);
      next?.delayIds.forEach((id) => consumedIds.add(id));
      currentNodeId = next?.nodeId;
    }

    if (textParts.length < 2) return undefined;
    const consumedNodes = [...consumedIds].map((id) => nodesById.get(id)).filter(Boolean) as Node[];
    if (consumedNodes.some((node) => nodeHasValueEdge(node.id))) return undefined;

    const firstNode = nodesById.get(startNodeId);
    const lastNode = nodesById.get(lastNodeId);
    if (!firstNode || !lastNode) return undefined;

    return {
      text: textParts.join(""),
      nodes: consumedNodes,
      consumedIds,
      incomingEdges: (incomingByTarget.get(firstNode.id) ?? []).filter(
        (edge) => !consumedIds.has(edge.source),
      ),
      outgoingEdges: (outgoingBySource.get(lastNode.id) ?? []).filter(
        (edge) => !consumedIds.has(edge.target),
      ),
    };
  }

  function nextPrintableKeyNode(
    fromNodeId: string,
    selectedIds: Set<string>,
    alreadyConsumed: Set<string>,
    currentConsumed: Set<string>,
    nodesById: Map<string, Node>,
    outgoingBySource: Map<string, Edge[]>,
  ) {
    const delayIds: string[] = [];
    let nextNodeId = singleOutgoingTarget(fromNodeId, outgoingBySource);

    while (nextNodeId) {
      if (!selectedIds.has(nextNodeId) || alreadyConsumed.has(nextNodeId) || currentConsumed.has(nextNodeId)) {
        return undefined;
      }

      const nextNode = nodesById.get(nextNodeId);
      if (!nextNode) return undefined;
      if (nextNode.type === "delayNode") {
        delayIds.push(nextNode.id);
        nextNodeId = singleOutgoingTarget(nextNode.id, outgoingBySource);
        continue;
      }

      return nextNode.type === "keyNode" && textForKeyNode(nextNode) !== undefined
        ? { nodeId: nextNode.id, delayIds }
        : undefined;
    }

    return undefined;
  }

  function textForKeyNode(node: Node) {
    const key = String(node.data.key ?? "");
    if (key.length === 1) return key;

    switch (key.toLowerCase()) {
      case "space":
        return " ";
      case "tab":
        return "\t";
      case "enter":
      case "return":
        return "\n";
      default:
        return undefined;
    }
  }

  function singleOutgoingTarget(nodeId: string, outgoingBySource: Map<string, Edge[]>) {
    const outgoing = outgoingBySource.get(nodeId) ?? [];
    return outgoing.length === 1 ? outgoing[0].target : undefined;
  }

  function nodeHasValueEdge(nodeId: string) {
    return edges.some(
      (edge) => edgeKind(edge) === "value" && (edge.source === nodeId || edge.target === nodeId),
    );
  }

  function groupEdges(edgeList: Edge[], key: "source" | "target") {
    const grouped = new Map<string, Edge[]>();
    for (const edge of edgeList) {
      const id = edge[key];
      const group = grouped.get(id) ?? [];
      group.push(edge);
      grouped.set(id, group);
    }
    return grouped;
  }

  function addReplacementEdge(
    replacementEdges: Edge[],
    edgeKeys: Set<string>,
    source: string,
    sourceHandle: string | null,
    target: string,
    targetHandle: string | null,
  ) {
    const edgeKey = `${source}:${sourceHandle ?? ""}->${target}:${targetHandle ?? ""}`;
    if (edgeKeys.has(edgeKey)) return;
    edgeKeys.add(edgeKey);
    replacementEdges.push(makeGraphEdge({ source, sourceHandle, target, targetHandle }));
  }

  function copySelection() {
    const copiedIds = selectedNodeIdsWithDescendants();
    if (copiedIds.size === 0) return;

    copiedGraph = {
      nodes: cloneData(nodes.filter((node) => copiedIds.has(node.id))),
      edges: cloneData(edges.filter((edge) => copiedIds.has(edge.source) && copiedIds.has(edge.target))),
    };
    pasteIndex = 0;
    closeContextMenu();
  }

  function graphBounds(copiedNodes: Node[]) {
    const rootNodes = copiedNodes.filter((node) => !node.parentId || !copiedNodes.some((copy) => copy.id === node.parentId));
    const candidates = rootNodes.length > 0 ? rootNodes : copiedNodes;
    const minX = Math.min(...candidates.map((node) => node.position.x));
    const minY = Math.min(...candidates.map((node) => node.position.y));

    return { minX, minY };
  }

  function pasteSelection(position?: { x: number; y: number }) {
    if (!copiedGraph || copiedGraph.nodes.length === 0) return;

    const copiedIds = new Set(copiedGraph.nodes.map((node) => node.id));
    const idMap = new Map(copiedGraph.nodes.map((node) => [node.id, crypto.randomUUID()]));
    const bounds = graphBounds(copiedGraph.nodes);
    const fallbackOffset = 36 * (pasteIndex + 1);
    const offset = position
      ? {
          x: position.x - bounds.minX,
          y: position.y - bounds.minY,
        }
      : {
          x: fallbackOffset,
          y: fallbackOffset,
        };

    const pastedNodes = copiedGraph.nodes.map((node) => {
      const nextNode = cloneData(node) as Node & {
        parentId?: string;
        extent?: Node["extent"];
      };
      nextNode.id = idMap.get(node.id) ?? crypto.randomUUID();
      nextNode.selected = true;

      if (node.parentId && copiedIds.has(node.parentId)) {
        nextNode.parentId = idMap.get(node.parentId);
      } else {
        delete nextNode.parentId;
        delete nextNode.extent;
        nextNode.position = {
          x: node.position.x + offset.x,
          y: node.position.y + offset.y,
        };
      }

      return nextNode;
    });

    const pastedEdges = copiedGraph.edges.map((edge) => ({
      ...cloneData(edge),
      id: `e-${idMap.get(edge.source)}-${idMap.get(edge.target)}-${crypto.randomUUID()}`,
      source: idMap.get(edge.source) ?? edge.source,
      target: idMap.get(edge.target) ?? edge.target,
      selected: false,
    }));

    nodes = [
      ...nodes.map((node) => ({ ...node, selected: false })),
      ...pastedNodes,
    ];
    edges = [
      ...edges.map((edge) => ({ ...edge, selected: false })),
      ...pastedEdges,
    ];
    selectedNodes = pastedNodes;
    pasteIndex += 1;
    closeContextMenu();
  }

  function deleteSelection() {
    const selectedIds = selectedNodeIdsWithDescendants();
    if (selectedIds.size === 0) return;

    nodes = nodes.filter((node) => !selectedIds.has(node.id));
    edges = edges.filter((edge) => !selectedIds.has(edge.source) && !selectedIds.has(edge.target));
    selectedNodes = [];
    closeContextMenu();
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (isEditingTarget(event.target)) return;

    const shortcut = event.ctrlKey || event.metaKey;
    if (!shortcut) return;

    const key = event.key.toLowerCase();

    if (key === "z" && event.shiftKey) {
      event.preventDefault();
      redo();
    } else if (key === "z") {
      event.preventDefault();
      undo();
    } else if (key === "y") {
      event.preventDefault();
      redo();
    } else if (key === "c") {
      event.preventDefault();
      copySelection();
    } else if (key === "v") {
      event.preventDefault();
      pasteSelection();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  });

  async function saveMacro() {
    try {
      // Get current flow state
      const flowData = toObject();
      const syncedSubmacros = syncSubmacroDefinitionsFromSubflows(
        flowData.nodes,
        flowData.edges,
        submacros,
      );
      submacros = syncedSubmacros;

      const projectData: MacroProjectData = {
        nodes: flowData.nodes,
        edges: flowData.edges,
        variables,
        submacros: syncedSubmacros,
        viewport: flowData.viewport,
      };

      // Open save dialog
      const filePath = await save({
        filters: [
          {
            name: "Macro JSON",
            extensions: ["json"],
          },
        ],
      });

      if (filePath) {
        await writeTextFile(filePath, JSON.stringify(projectData, null, 2));
        console.log("Macro saved to:", filePath);
      }
    } catch (error) {
      console.error("Failed to save macro:", error);
    }
  }

  async function loadMacro() {
    try {
      const filePath = await open({
        multiple: false,
        filters: [
          {
            name: "Macro JSON",
            extensions: ["json"],
          },
        ],
      });

      if (filePath && typeof filePath === "string") {
        const content = await readTextFile(filePath);
        const flowData = JSON.parse(content);

        if (flowData.nodes && flowData.edges) {
          nodes = flowData.nodes;
          edges = flowData.edges;
          variables = flowData.variables ?? [];
          submacros = flowData.submacros ?? [];

          if (flowData.viewport) {
            const { x, y, zoom } = flowData.viewport;
            setViewport({ x, y, zoom });
          }
          console.log("Macro loaded from:", filePath);
        }
      }
    } catch (error) {
      console.error("Failed to load macro:", error);
    }
  }

  function onDragOver(event: DragEvent) {
    event.preventDefault();
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }

  function nodeDimension(node: (typeof nodes)[number], property: "width" | "height") {
    const measured = node as (typeof nodes)[number] & {
      measured?: {
        width?: number;
        height?: number;
      };
      width?: number;
      height?: number;
    };

    if (measured[property]) return measured[property] ?? 0;
    if (measured.measured?.[property]) return measured.measured[property] ?? 0;

    const styleValue = typeof node.style === "string"
      ? node.style.match(new RegExp(`${property}:\\s*(\\d+(?:\\.\\d+)?)px`))?.[1]
      : undefined;

    return Number(styleValue ?? (property === "width" ? 420 : 260));
  }

  function findSubflowGroupAt(position: { x: number; y: number }) {
    return [...nodes].reverse().find((node) => {
      if (node.type !== "subflowGroupNode") return false;

      const width = nodeDimension(node, "width");
      const height = nodeDimension(node, "height");

      return (
        position.x >= node.position.x &&
        position.x <= node.position.x + width &&
        position.y >= node.position.y &&
        position.y <= node.position.y + height
      );
    });
  }

  function onDrop(event: DragEvent) {
    event.preventDefault();
    if (!event.dataTransfer) return;

    const data = event.dataTransfer.getData("application/svelteflow");
    if (!data) return;

    const { type, data: nodeData } = JSON.parse(data);

    const position = screenToFlowPosition({
      x: event.clientX,
      y: event.clientY,
    });
    const parentSubflow = findSubflowGroupAt(position);

    const newNode = {
      id: crypto.randomUUID(),
      type,
      position: parentSubflow
        ? {
            x: position.x - parentSubflow.position.x,
            y: position.y - parentSubflow.position.y,
          }
        : position,
      data: nodeData,
      origin: [0.5, 0.5] as [number, number],
      parentId: parentSubflow?.id,
      extent: parentSubflow ? ("parent" as const) : undefined,
    };

    nodes = [...nodes, newNode];
  }

  function handleSelectionChange(selection: { nodes: typeof nodes; edges: typeof edges }) {
    selectedNodes = selection.nodes;
  }

  function openContextMenu(
    event: MouseEvent,
    target: ContextMenuState["target"],
  ) {
    event.preventDefault();
    event.stopPropagation();

    contextMenu = {
      x: event.clientX,
      y: event.clientY,
      flowPosition: screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      }),
      target,
      delayBaselines: selectedDelayBaselines(),
      delayMultiplier: 1,
    };
  }

  function handlePaneContextMenu({ event }: { event: MouseEvent }) {
    openContextMenu(event, "pane");
  }

  function handleSelectionContextMenu({ event }: { event: MouseEvent; nodes: Node[] }) {
    openContextMenu(event, "selection");
  }

  function handleNodeContextMenu({ event, node }: { event: MouseEvent; node: Node }) {
    if (!selectedNodes.some((selectedNode) => selectedNode.id === node.id)) {
      nodes = nodes.map((candidate) => ({
        ...candidate,
        selected: candidate.id === node.id,
      }));
      selectedNodes = [node];
    }

    openContextMenu(event, "node");
  }

  function createSubmacroFromSelection() {
    const result = createSubmacroFromSelectionModel({
      nodes,
      edges,
      selectedNodes,
      submacroCount: submacros.length,
    });
    if (!result) return;

    submacros = [...submacros, result.submacro];
    nodes = result.nodes;
    edges = result.edges;
    selectedNodes = [];
    closeContextMenu();
  }

  function addSubmacroInstance(definition: SubmacroDefinition) {
    const position = screenToFlowPosition({
      x: window.innerWidth / 2,
      y: window.innerHeight / 2,
    });

    nodes = [
      ...nodes,
      {
        id: crypto.randomUUID(),
        type: "submacroNode",
        position,
        data: submacroNodeData(definition),
      },
    ];
  }
</script>

<div
  class="flow-container"
  role="application"
  ondragover={onDragOver}
  ondrop={onDrop}
>
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    {defaultEdgeOptions}
    {isValidConnection}
    onbeforeconnect={onBeforeConnect}
    onselectionchange={handleSelectionChange}
    onpanecontextmenu={handlePaneContextMenu}
    onselectioncontextmenu={handleSelectionContextMenu}
    onnodecontextmenu={handleNodeContextMenu}
    onpaneclick={closeContextMenu}
    fitView
    colorMode="dark"
    proOptions={{ hideAttribution: true }}
    zoomOnDoubleClick={false}
  >
    <Background
      variant={BackgroundVariant.Lines}
      bgColor="#121212"
      patternColor="#242424"
      gap={25}
    />
    <!-- SVG definitions for gradient and markers -->
    <svg>
      <defs>
        <linearGradient id="edge-gradient">
          <stop offset="0%" stop-color="#e92a67" />
          <stop offset="25%" stop-color="#ae53ba" />
          <stop offset="75%" stop-color="#2a8af6" />
          <stop offset="100%" stop-color="#e92a67" />
        </linearGradient>
      </defs>
    </svg>
    <Controls />
    <Panel position="top-right">
      <button class="panel-btn" disabled={undoStack.length === 0} onclick={undo}>
        Undo
      </button>
      <button class="panel-btn" disabled={redoStack.length === 0} onclick={redo}>
        Redo
      </button>
      <select
        class="panel-select"
        bind:value={recordingMode}
        disabled={isRecording}
        aria-label="Recording mode"
      >
        <option value="movesBeforeClicks">Clicks only</option>
        <option value="allMoves">All movement</option>
      </select>
      <button
        class="panel-btn record-btn"
        class:recording={isRecording}
        disabled={isRunning}
        onclick={toggleRecording}
      >
        {isRecording ? "Stop Recording" : "Record"}
      </button>
      <button class="panel-btn" disabled={isRecording} onclick={toggleExecution}>
        {isRunning ? "Stop" : "Run"}
      </button>
      <button class="panel-btn" onclick={() => (showVariables = !showVariables)}>
        Variables
      </button>
      <button
        class="panel-btn"
        disabled={selectedNodes.length === 0}
        onclick={createSubmacroFromSelection}
      >
        Create Subflow
      </button>
      <button class="panel-btn" onclick={saveMacro}>Save</button>
      <button class="panel-btn" onclick={loadMacro}>Load</button>
    </Panel>
    <Panel position="top-left">
      {#if showVariables}
        <VariablesPanel bind:variables snapshot={variableSnapshot} />
      {/if}
      {#if submacros.length > 0}
        <div class="submacro-panel">
          <div class="submacro-title">Subflow Calls</div>
          {#each submacros as submacro (submacro.id)}
            <button class="submacro-btn" onclick={() => addSubmacroInstance(submacro)}>
              {submacro.name}
            </button>
          {/each}
        </div>
      {/if}
    </Panel>
    <MiniMap />
  </SvelteFlow>

  {#if recordingError}
    <div class="recording-error" role="status">
      {recordingError}
    </div>
  {/if}

  {#if contextMenu}
    <div
      class="context-menu"
      style={`left: ${contextMenu.x}px; top: ${contextMenu.y}px;`}
      role="menu"
      tabindex="-1"
      oncontextmenu={(event) => event.preventDefault()}
    >
      <button role="menuitem" disabled={undoStack.length === 0} onclick={undo}>Undo</button>
      <button role="menuitem" disabled={redoStack.length === 0} onclick={redo}>Redo</button>
      <div class="menu-separator"></div>
      <button
        role="menuitem"
        disabled={selectedNodes.length === 0}
        onclick={copySelection}
      >
        Copy
      </button>
      <button
        role="menuitem"
        disabled={!copiedGraph}
        onclick={() => pasteSelection(contextMenu?.flowPosition)}
      >
        Paste
      </button>
      <button
        role="menuitem"
        disabled={selectedNodes.length === 0}
        onclick={createSubmacroFromSelection}
      >
        Create Subflow
      </button>
      <div class="menu-separator"></div>
      <button
        role="menuitem"
        disabled={!canOptimizeTypingTextSelection()}
        onclick={optimizeTypingTextSelection}
      >
        Optimize Typing Text
      </button>
      <button
        role="menuitem"
        disabled={selectedDelayCount() === 0}
        onclick={deleteSelectedDelayNodes}
      >
        Delete Delay Nodes
      </button>
      <div
        class="delay-multiplier"
        class:disabled={selectedDelayCount() === 0}
        role="group"
        aria-label="Delay multiplier"
      >
        <div class="delay-multiplier-header">
          <span>Delay Multiplier</span>
          <span>{Math.round(contextMenu.delayMultiplier * 100)}%</span>
        </div>
        <input
          type="range"
          min="0"
          max="2"
          step="0.05"
          value={contextMenu.delayMultiplier}
          disabled={selectedDelayCount() === 0}
          aria-label="Scale selected delay values"
          oninput={(event) => applyDelayMultiplier(Number(event.currentTarget.value))}
        />
        <div class="delay-multiplier-scale">
          <span>0%</span>
          <span>100%</span>
          <span>200%</span>
        </div>
      </div>
      <div class="menu-separator"></div>
      <button
        role="menuitem"
        class="danger"
        disabled={selectedNodes.length === 0}
        onclick={deleteSelection}
      >
        Delete
      </button>
    </div>
  {/if}

  {#if pendingRecordedMacro}
    <div class="recording-dialog-backdrop">
      <div
        class="recording-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="recording-dialog-title"
      >
        <h2 id="recording-dialog-title">Set Keybind</h2>
        <div class="recording-summary">
          {pendingRecordedMacro.actions.length} actions recorded
        </div>
        <button
          class="shortcut-capture"
          class:recording={recordingShortcut}
          onkeydown={handleRecordedShortcutKeyDown}
          onclick={startShortcutRecording}
          onblur={() => {
            if (recordingShortcut) {
              recordingShortcut = false;
              recordedShortcut = shortcutReady() ? recordedShortcut : "";
            }
          }}
        >
          {recordedShortcut && recordedShortcut !== "..." ? recordedShortcut : "Record keybind"}
        </button>
        <div class="dialog-actions">
          <button class="panel-btn" onclick={discardRecordedMacro}>Discard</button>
          <button class="panel-btn" onclick={() => createRecordedMacro("")}>
            Skip Keybind
          </button>
          <button
            class="panel-btn primary"
            disabled={!shortcutReady()}
            onclick={() => createRecordedMacro()}
          >
            Create Macro
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .flow-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
  }

  .panel-btn {
    background: #2c2d2f;
    color: #e0e0e0;
    border: 1px solid #3e3e3e;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    transition: all 0.2s;
    margin-left: 8px;
  }

  .panel-btn:hover {
    background: #38393c;
    border-color: #555;
    color: #fff;
  }

  .panel-btn.primary {
    background: #2a8af6;
    border-color: #4ba0ff;
    color: #fff;
  }

  .panel-btn.record-btn.recording {
    background: #3a2029;
    border-color: #e92a67;
    color: #fff;
  }

  .panel-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
  }

  .panel-select {
    background: #232426;
    border: 1px solid #3e3e3e;
    border-radius: 4px;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
    margin-left: 8px;
    min-height: 29px;
    padding: 5px 28px 5px 8px;
  }

  .panel-select:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .recording-error {
    background: rgba(58, 32, 41, 0.96);
    border: 1px solid #e92a67;
    border-radius: 6px;
    color: #ffd7e2;
    font-size: 0.8rem;
    left: 50%;
    max-width: min(460px, calc(100vw - 2rem));
    padding: 0.55rem 0.75rem;
    position: fixed;
    top: 64px;
    transform: translateX(-50%);
    z-index: 30;
  }

  .submacro-panel {
    background: rgba(18, 18, 18, 0.96);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    color: #e0e0e0;
    display: grid;
    gap: 0.35rem;
    margin-top: 0.5rem;
    min-width: 180px;
    padding: 0.65rem;
  }

  .submacro-title {
    color: #888;
    font-size: 0.72rem;
    text-transform: uppercase;
  }

  .submacro-btn {
    background: #2c2d2f;
    border: 1px solid #414141;
    border-radius: 4px;
    color: #f1f1f1;
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
    padding: 0.35rem 0.45rem;
    text-align: left;
  }

  .context-menu {
    background: rgba(18, 18, 18, 0.98);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    box-shadow: 0 14px 32px rgba(0, 0, 0, 0.36);
    display: grid;
    gap: 0.15rem;
    min-width: 168px;
    padding: 0.3rem;
    position: fixed;
    z-index: 20;
  }

  .context-menu button {
    background: transparent;
    border: none;
    border-radius: 4px;
    color: #f1f1f1;
    cursor: pointer;
    font: inherit;
    font-size: 0.78rem;
    padding: 0.38rem 0.5rem;
    text-align: left;
  }

  .context-menu button:hover:not(:disabled) {
    background: #2c2d2f;
  }

  .context-menu button:disabled {
    color: #686868;
    cursor: not-allowed;
  }

  .context-menu button.danger {
    color: #ff8aa8;
  }

  .delay-multiplier {
    display: grid;
    gap: 0.3rem;
    padding: 0.35rem 0.5rem 0.4rem;
  }

  .delay-multiplier.disabled {
    opacity: 0.45;
  }

  .delay-multiplier-header,
  .delay-multiplier-scale {
    align-items: center;
    color: #b8b8b8;
    display: flex;
    font-size: 0.72rem;
    justify-content: space-between;
  }

  .delay-multiplier-header span:first-child {
    color: #f1f1f1;
  }

  .delay-multiplier input[type="range"] {
    accent-color: #a853ba;
    cursor: pointer;
    margin: 0;
    width: 100%;
  }

  .delay-multiplier input[type="range"]:disabled {
    cursor: not-allowed;
  }

  .menu-separator {
    background: #303030;
    height: 1px;
    margin: 0.15rem 0;
  }

  .recording-dialog-backdrop {
    align-items: center;
    background: rgba(0, 0, 0, 0.42);
    display: flex;
    inset: 0;
    justify-content: center;
    position: fixed;
    z-index: 40;
  }

  .recording-dialog {
    background: #18191b;
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.45);
    color: #f1f1f1;
    display: grid;
    gap: 0.75rem;
    max-width: min(360px, calc(100vw - 2rem));
    padding: 1rem;
    width: 100%;
  }

  .recording-dialog h2 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
  }

  .recording-summary {
    color: #9da3ad;
    font-size: 0.82rem;
  }

  .shortcut-capture {
    align-items: center;
    background: #2c2d2f;
    border: 1px solid #555;
    border-radius: 4px;
    color: #e0e0e0;
    cursor: pointer;
    display: flex;
    font-family: "Fira Mono", monospace;
    font-size: 0.9rem;
    justify-content: center;
    min-height: 40px;
    overflow: hidden;
    padding: 0.45rem 0.7rem;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .shortcut-capture:focus,
  .shortcut-capture.recording {
    border-color: #e92a67;
    box-shadow: 0 0 0 1px rgba(233, 42, 103, 0.3);
    outline: none;
  }

  .dialog-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .dialog-actions .panel-btn {
    margin-left: 0;
  }
</style>
