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
    type OnBeforeDelete,
    type OnConnectEnd,
    useEdges,
    useNodes,
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
    FLOW_IN_HANDLE,
    FLOW_OUT_HANDLE,
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
  import { writeTextFile, readTextFile } from "@tauri-apps/plugin-fs";
  import { listen as listenEvent } from "@tauri-apps/api/event";
  import {
    recordedMacroToGraph,
    type RecordedMacro,
    type RecordingMouseMode,
  } from "$lib/recording";
  import { MacroRunner } from "$lib/runner/MacroRunner";
  import { nodeTemplates, type NodeTemplate } from "$lib/node-palette";
  import {
    MODIFIER_KEYS,
    modifierShortcutPreview,
    shortcutFromKeyboardEvent,
  } from "$lib/shortcuts";
  import { onDestroy, onMount, setContext, tick } from "svelte";

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
    canOptimizeTypingText: boolean;
  };

  type EdgeDropMenuState = {
    x: number;
    y: number;
    flowPosition: { x: number; y: number };
    sourceNodeId?: string;
    sourceHandle?: string | null;
  };

  type MousePositionPayload = {
    x: number;
    y: number;
    clicked: boolean;
    button?: string | null;
  };

  type SavedMouseClick = {
    id: string;
    x: number;
    y: number;
    button?: string | null;
    createdAt: string;
  };

  const HISTORY_LIMIT = 80;
  const MAX_SAVED_MOUSE_CLICKS = 24;

  // use $state.raw for performance as recommended by xyflow docs
  let nodes = $state.raw(initialNodes);
  let edges = $state.raw(initialEdges);
  let variables = $state.raw([] as VariableDefinition[]);
  let submacros = $state.raw([] as SubmacroDefinition[]);
  let selectedNodes = $state.raw([] as typeof nodes);
  let variableSnapshot = $state({} as Record<string, MacroValue>);
  let showVariables = $state(false);
  let showMousePositionPanel = $state(false);
  let isMousePositionMonitoring = $state(false);
  let mousePositionError = $state("");
  let currentMousePosition = $state<MousePositionPayload | undefined>();
  let savedMouseClicks = $state.raw([] as SavedMouseClick[]);
  let undoStack = $state.raw([] as EditorSnapshot[]);
  let redoStack = $state.raw([] as EditorSnapshot[]);
  let copiedGraph = $state.raw<GraphClipboard | undefined>();
  let contextMenu = $state<ContextMenuState | undefined>();
  let edgeDropMenu = $state<EdgeDropMenuState | undefined>();
  let edgeDropQuery = $state("");
  let edgeDropSearchInput = $state<HTMLInputElement | undefined>();
  let cachedEdgeDropQuery: string | undefined;
  let cachedEdgeDropTemplates = nodeTemplates;
  let ignoreNextPaneClick = false;
  let pasteIndex = 0;
  let applyingHistory = false;
  let historyTimer: ReturnType<typeof setTimeout> | undefined;
  let pendingMousePosition: MousePositionPayload | undefined;
  let mousePositionFrame: number | undefined;
  let savedMouseClickId = 0;
  let savedMouseClickVersion = $state(0);

  const valueSourceContext: ValueSourceContext = {
    getVariables: () => variables,
    getSnapshot: () => variableSnapshot,
  };

  const renameSubflowDefinition: RenameSubflow = (definitionId, rawName) => {
    const trimmedName = rawName.trim();
    const name = trimmedName || "Subflow";
    const now = new Date().toISOString();

    for (const submacro of submacros) {
      if (submacro.id !== definitionId) continue;
      submacro.name = name;
      submacro.updatedAt = now;
      break;
    }
    submacros = submacros;

    for (const node of nodes) {
      const data = node.data as { definitionId?: string } | undefined;
      if (
        (node.type === "subflowGroupNode" || node.type === "submacroNode") &&
        data?.definitionId === definitionId
      ) {
        node.data.title = rawName || name;
      }
    }
    commitNodes();
  };

  setContext(SUBFLOW_RENAME_CONTEXT, renameSubflowDefinition);
  setContext(VALUE_SOURCE_CONTEXT, valueSourceContext);

  function clearNodeSelection(nodeList: Node[]) {
    let changed = false;
    for (const node of nodeList) {
      if (!node.selected) continue;
      node.selected = false;
      changed = true;
    }
    return changed;
  }

  function clearEdgeSelection(edgeList: Edge[]) {
    let changed = false;
    for (const edge of edgeList) {
      if (!edge.selected) continue;
      edge.selected = false;
      changed = true;
    }
    return changed;
  }

  function edgeKey(
    source: string,
    sourceHandle: string | null | undefined,
    target: string,
    targetHandle: string | null | undefined,
  ) {
    return `${source}:${sourceHandle ?? ""}->${target}:${targetHandle ?? ""}`;
  }

  function commitNodes() {
    nodes = nodes;
    flowNodes.set(nodes.slice());
  }

  function commitEdges() {
    edges = edges;
    flowEdges.set(edges.slice());
  }

  function currentEditorSnapshot(): EditorSnapshot {
    return {
      nodes: structuredClone(nodes),
      edges: structuredClone(edges),
      variables: structuredClone(variables),
      submacros: structuredClone(submacros),
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

    undoStack.push(lastHistorySnapshot);
    if (undoStack.length > HISTORY_LIMIT) undoStack.shift();
    undoStack = undoStack;
    redoStack.length = 0;
    redoStack = redoStack;
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
    nodes = structuredClone(snapshot.nodes);
    edges = structuredClone(snapshot.edges);
    variables = structuredClone(snapshot.variables);
    submacros = structuredClone(snapshot.submacros);
    selectedNodes.length = 0;
    selectedNodes = selectedNodes;
    lastHistorySnapshot = structuredClone(snapshot);
    lastHistorySignature = snapshotSignature(snapshot);
    queueMicrotask(() => {
      applyingHistory = false;
    });
  }

  function undo() {
    flushHistoryCommit();
    const snapshot = undoStack[undoStack.length - 1];
    if (!snapshot) return;

    const current = currentEditorSnapshot();
    undoStack.pop();
    undoStack = undoStack;
    redoStack.unshift(current);
    if (redoStack.length > HISTORY_LIMIT) redoStack.length = HISTORY_LIMIT;
    redoStack = redoStack;
    restoreSnapshot(snapshot);
    closeContextMenu();
  }

  function redo() {
    flushHistoryCommit();
    const snapshot = redoStack[0];
    if (!snapshot) return;

    const current = currentEditorSnapshot();
    redoStack.shift();
    redoStack = redoStack;
    undoStack.push(current);
    if (undoStack.length > HISTORY_LIMIT) undoStack.shift();
    undoStack = undoStack;
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

  const onConnectEnd: OnConnectEnd = (event, connectionState) => {
    if (connectionState.isValid) return;

    const sourceNodeId = connectionState.fromNode?.id;
    const sourceHandle = connectionState.fromHandle?.id ?? null;
    if (!sourceNodeId || isValueHandle(sourceHandle)) {
      return;
    }

    const position = pointerClientPosition(event);
    if (!position) return;

    edgeDropQuery = "";
    edgeDropMenu = {
      x: position.x,
      y: position.y,
      flowPosition: screenToFlowPosition(position),
      sourceNodeId,
      sourceHandle,
    };
    ignoreNextPaneClick = true;
    contextMenu = undefined;
    showRecordOptions = false;
    void tick().then(() => edgeDropSearchInput?.focus());
  };

  const onBeforeDelete: OnBeforeDelete = async ({ nodes: deletedNodes }) => {
    const deletedIds = new Set<string>();
    for (const node of deletedNodes) {
      deletedIds.add(node.id);
    }
    if (deletedIds.size === 0) return true;

    deleteNodesWithBypass(deletedIds);
    return true;
  };

  // DnD Hook
  const { screenToFlowPosition, toObject, setViewport } = useSvelteFlow();
  const flowNodes = useNodes();
  const flowEdges = useEdges();

  let runner: MacroRunner | undefined = $state();
  let isRunning = $state(false);
  let isRecording = $state(false);
  let recordingMode = $state<RecordingMouseMode>("movesBeforeClicks");
  let showRecordOptions = $state(false);
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
          variableSnapshot = snapshot;
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
      showRecordOptions = !showRecordOptions;
    }
  }

  async function startRecordingWithMode(mode: RecordingMouseMode) {
    recordingMode = mode;
    showRecordOptions = false;
    await startRecording();
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

    const selectedRecordedNodes = new Array<Node>(graph.nodes.length);
    for (let index = 0; index < graph.nodes.length; index += 1) {
      selectedRecordedNodes[index] = {
        ...graph.nodes[index],
        selected: true,
      };
    }

    clearNodeSelection(nodes);
    for (const node of selectedRecordedNodes) {
      nodes.push(node);
    }
    commitNodes();
    for (const edge of graph.edges) {
      edges.push(edge);
    }
    commitEdges();
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

  async function toggleMousePositionMonitoring() {
    mousePositionError = "";
    try {
      if (isMousePositionMonitoring) {
        await invoke("stop_mouse_position_monitor");
        isMousePositionMonitoring = false;
      } else {
        await invoke("start_mouse_position_monitor");
        isMousePositionMonitoring = true;
      }
    } catch (error) {
      mousePositionError = String(error);
      console.error("Failed to toggle mouse position monitor:", error);
    }
  }

  function clearSavedMouseClicks() {
    savedMouseClicks.length = 0;
    savedMouseClicks = savedMouseClicks;
    savedMouseClickVersion += 1;
  }

  function scheduleMousePositionUpdate(position: MousePositionPayload) {
    pendingMousePosition = position;
    if (mousePositionFrame !== undefined) return;

    mousePositionFrame = requestAnimationFrame(() => {
      mousePositionFrame = undefined;
      if (!pendingMousePosition) return;

      currentMousePosition = pendingMousePosition;
      pendingMousePosition = undefined;
    });
  }

  function saveMouseClick(position: MousePositionPayload) {
    savedMouseClickId += 1;
    savedMouseClicks.unshift({
      id: `${savedMouseClickId}`,
      x: position.x,
      y: position.y,
      button: position.button,
      createdAt: new Date().toLocaleTimeString(),
    });
    if (savedMouseClicks.length > MAX_SAVED_MOUSE_CLICKS) {
      savedMouseClicks.length = MAX_SAVED_MOUSE_CLICKS;
    }
    savedMouseClicks = savedMouseClicks;
    savedMouseClickVersion += 1;
  }

  onDestroy(() => {
    if (historyTimer) {
      clearTimeout(historyTimer);
    }

    if (mousePositionFrame !== undefined) {
      cancelAnimationFrame(mousePositionFrame);
    }

    if (runner) {
      runner.cleanup();
    }

    if (isRecording) {
      void invoke("stop_macro_recording");
    }

    if (isMousePositionMonitoring) {
      void invoke("stop_mouse_position_monitor");
    }
  });

  function closeContextMenu() {
    contextMenu = undefined;
    edgeDropMenu = undefined;
    edgeDropQuery = "";
    showRecordOptions = false;
  }

  function isEditingTarget(target: EventTarget | null) {
    if (!(target instanceof HTMLElement)) return false;
    return Boolean(target.closest("input, textarea, select, [contenteditable='true']"));
  }

  function selectedNodeIdsWithDescendants() {
    const copiedIds = new Set<string>();
    for (const node of selectedNodes) {
      copiedIds.add(node.id);
    }
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

    const triggerEdges: Edge[] = [];
    const outgoingBySource = new Map<string, Edge[]>();
    const keptEdges: Edge[] = [];
    const edgeKeys = new Set<string>();
    const bypassEdges: Edge[] = [];

    for (const edge of edges) {
      const isTrigger = edgeKind(edge) === "trigger";
      if (isTrigger) {
        triggerEdges.push(edge);
        const sourceEdges = outgoingBySource.get(edge.source);
        if (sourceEdges) {
          sourceEdges.push(edge);
        } else {
          outgoingBySource.set(edge.source, [edge]);
        }
      }

      if (!delayIds.has(edge.source) && !delayIds.has(edge.target)) {
        keptEdges.push(edge);
        edgeKeys.add(edgeKey(edge.source, edge.sourceHandle, edge.target, edge.targetHandle));
      }
    }

    function reachableOutputs(nodeId: string, visited = new Set<string>()): Edge[] {
      if (visited.has(nodeId)) return [];
      visited.add(nodeId);

      const outgoing = outgoingBySource.get(nodeId) ?? [];
      const outputs: Edge[] = [];

      for (const edge of outgoing) {
        if (delayIds.has(edge.target)) {
          for (const output of reachableOutputs(edge.target, visited)) {
            outputs.push(output);
          }
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

        const key = edgeKey(
          incomingEdge.source,
          incomingEdge.sourceHandle,
          outgoingEdge.target,
          outgoingEdge.targetHandle,
        );
        if (edgeKeys.has(key)) continue;
        edgeKeys.add(key);

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

    let nodeWriteIndex = 0;
    for (const node of nodes) {
      if (delayIds.has(node.id)) continue;
      nodes[nodeWriteIndex] = node;
      nodeWriteIndex += 1;
    }
    nodes.length = nodeWriteIndex;
    commitNodes();
    for (const edge of bypassEdges) {
      keptEdges.push(edge);
    }
    edges = keptEdges;
    let selectedWriteIndex = 0;
    for (const node of selectedNodes) {
      if (delayIds.has(node.id)) continue;
      selectedNodes[selectedWriteIndex] = node;
      selectedWriteIndex += 1;
    }
    selectedNodes.length = selectedWriteIndex;
    selectedNodes = selectedNodes;
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

    for (const node of nodes) {
      const baseline = baselines[node.id];
      if (baseline === undefined || node.type !== "delayNode") {
        continue;
      }
      node.data.delay = Math.round(baseline * multiplier);
    }
    commitNodes();
  }

  function optimizeTypingTextSelection() {
    const runs = findTypingOptimizationRuns();
    if (runs.length === 0) return;

    const consumedIds = new Set<string>();
    for (const run of runs) {
      for (const id of run.consumedIds) {
        consumedIds.add(id);
      }
    }

    const replacementNodes = new Array<Node>(runs.length);
    for (let index = 0; index < runs.length; index += 1) {
      const run = runs[index];
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

      replacementNodes[index] = replacement;
    }

    const keptEdges: Edge[] = [];
    const edgeKeys = new Set<string>();
    for (const edge of edges) {
      if (consumedIds.has(edge.source) || consumedIds.has(edge.target)) continue;
      keptEdges.push(edge);
      edgeKeys.add(edgeKey(edge.source, edge.sourceHandle, edge.target, edge.targetHandle));
    }
    const replacementEdges: Edge[] = [];

    for (let index = 0; index < runs.length; index += 1) {
      const run = runs[index];
      const replacement = replacementNodes[index];

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

    let nodeWriteIndex = 0;
    for (const node of nodes) {
      if (consumedIds.has(node.id)) continue;
      if (node.selected) node.selected = false;
      nodes[nodeWriteIndex] = node;
      nodeWriteIndex += 1;
    }
    nodes.length = nodeWriteIndex;
    for (const replacement of replacementNodes) {
      nodes.push(replacement);
    }

    commitNodes();
    for (const edge of replacementEdges) {
      keptEdges.push(edge);
    }
    edges = keptEdges;
    selectedNodes.length = 0;
    for (const replacement of replacementNodes) {
      selectedNodes.push(replacement);
    }
    selectedNodes = selectedNodes;
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

    const nodesById = new Map<string, Node>();
    for (const node of nodes) {
      nodesById.set(node.id, node);
    }

    const triggerEdges: Edge[] = [];
    const valueConnectedIds = new Set<string>();
    for (const edge of edges) {
      if (edgeKind(edge) === "trigger") {
        triggerEdges.push(edge);
      } else {
        valueConnectedIds.add(edge.source);
        valueConnectedIds.add(edge.target);
      }
    }
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
        valueConnectedIds,
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
    valueConnectedIds: Set<string>,
  ): TypingRun | undefined {
    const consumedIds = new Set<string>();
    const textParts: string[] = [];
    let currentNodeId: string | undefined = startNodeId;
    let lastNodeId = startNodeId;

    while (currentNodeId) {
      if (
        !selectedIds.has(currentNodeId) ||
        alreadyConsumed.has(currentNodeId) ||
        consumedIds.has(currentNodeId)
      ) {
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

      const next = nextPrintableKeyNode(
        node.id,
        selectedIds,
        alreadyConsumed,
        consumedIds,
        nodesById,
        outgoingBySource,
      );
      next?.delayIds.forEach((id) => consumedIds.add(id));
      currentNodeId = next?.nodeId;
    }

    if (textParts.length < 2) return undefined;
    const consumedNodes: Node[] = [];
    for (const id of consumedIds) {
      if (valueConnectedIds.has(id)) return undefined;
      const node = nodesById.get(id);
      if (node) consumedNodes.push(node);
    }

    const firstNode = nodesById.get(startNodeId);
    const lastNode = nodesById.get(lastNodeId);
    if (!firstNode || !lastNode) return undefined;

    return {
      text: textParts.join(""),
      nodes: consumedNodes,
      consumedIds,
      incomingEdges: unconsumedEdges(incomingByTarget.get(firstNode.id), consumedIds, "source"),
      outgoingEdges: unconsumedEdges(outgoingBySource.get(lastNode.id), consumedIds, "target"),
    };
  }

  function unconsumedEdges(
    edgeList: Edge[] | undefined,
    consumedIds: Set<string>,
    endpoint: "source" | "target",
  ) {
    const unconsumed: Edge[] = [];
    if (!edgeList) return unconsumed;

    for (const edge of edgeList) {
      if (!consumedIds.has(edge[endpoint])) unconsumed.push(edge);
    }
    return unconsumed;
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
      if (
        !selectedIds.has(nextNodeId) ||
        alreadyConsumed.has(nextNodeId) ||
        currentConsumed.has(nextNodeId)
      ) {
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

  function groupEdges(edgeList: Edge[], key: "source" | "target") {
    const grouped = new Map<string, Edge[]>();
    for (const edge of edgeList) {
      const id = edge[key];
      const group = grouped.get(id);
      if (group) {
        group.push(edge);
      } else {
        grouped.set(id, [edge]);
      }
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
    const key = edgeKey(source, sourceHandle, target, targetHandle);
    if (edgeKeys.has(key)) return;
    edgeKeys.add(key);
    replacementEdges.push(makeGraphEdge({ source, sourceHandle, target, targetHandle }));
  }

  function copySelection() {
    const copiedIds = selectedNodeIdsWithDescendants();
    if (copiedIds.size === 0) return;

    const copiedNodes: Node[] = [];
    const copiedEdges: Edge[] = [];
    for (const node of nodes) {
      if (copiedIds.has(node.id)) copiedNodes.push(node);
    }
    for (const edge of edges) {
      if (copiedIds.has(edge.source) && copiedIds.has(edge.target)) {
        copiedEdges.push(edge);
      }
    }

    copiedGraph = {
      nodes: structuredClone(copiedNodes),
      edges: structuredClone(copiedEdges),
    };
    pasteIndex = 0;
    closeContextMenu();
  }

  function graphBounds(copiedNodes: Node[]) {
    const copiedIds = new Set<string>();
    for (const node of copiedNodes) {
      copiedIds.add(node.id);
    }

    let minX = Number.POSITIVE_INFINITY;
    let minY = Number.POSITIVE_INFINITY;
    let rootCount = 0;
    for (const node of copiedNodes) {
      if (node.parentId && copiedIds.has(node.parentId)) continue;

      rootCount += 1;
      minX = Math.min(minX, node.position.x);
      minY = Math.min(minY, node.position.y);
    }

    if (rootCount === 0) {
      for (const node of copiedNodes) {
        minX = Math.min(minX, node.position.x);
        minY = Math.min(minY, node.position.y);
      }
    }

    return { minX, minY };
  }

  function pasteSelection(position?: { x: number; y: number }) {
    if (!copiedGraph || copiedGraph.nodes.length === 0) return;

    const copiedIds = new Set<string>();
    const idMap = new Map<string, string>();
    for (const node of copiedGraph.nodes) {
      copiedIds.add(node.id);
      idMap.set(node.id, crypto.randomUUID());
    }
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

    const pastedNodes = new Array<Node>(copiedGraph.nodes.length);
    for (let index = 0; index < copiedGraph.nodes.length; index += 1) {
      const node = copiedGraph.nodes[index];
      const nextNode = structuredClone(node) as Node & {
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

      pastedNodes[index] = nextNode;
    }

    const pastedEdges = new Array<Edge>(copiedGraph.edges.length);
    for (let index = 0; index < copiedGraph.edges.length; index += 1) {
      const edge = copiedGraph.edges[index];
      pastedEdges[index] = {
        ...structuredClone(edge),
        id: `e-${idMap.get(edge.source)}-${idMap.get(edge.target)}-${crypto.randomUUID()}`,
        source: idMap.get(edge.source) ?? edge.source,
        target: idMap.get(edge.target) ?? edge.target,
        selected: false,
      };
    }

    clearNodeSelection(nodes);
    for (const node of pastedNodes) {
      nodes.push(node);
    }
    commitNodes();
    clearEdgeSelection(edges);
    for (const edge of pastedEdges) {
      edges.push(edge);
    }
    commitEdges();
    selectedNodes = pastedNodes;
    pasteIndex += 1;
    closeContextMenu();
  }

  function deleteSelection() {
    const selectedIds = selectedNodeIdsWithDescendants();
    if (selectedIds.size === 0) return;

    deleteNodesWithBypass(selectedIds);
    selectedNodes.length = 0;
    selectedNodes = selectedNodes;
    closeContextMenu();
  }

  function deleteNodesWithBypass(deletedIds: Set<string>) {
    const keptEdges: Edge[] = [];
    const edgeKeys = new Set<string>();
    const incomingByDeletedTarget = new Map<string, Edge[]>();
    const outgoingByDeletedSource = new Map<string, Edge[]>();

    for (const edge of edges) {
      const sourceDeleted = deletedIds.has(edge.source);
      const targetDeleted = deletedIds.has(edge.target);

      if (!sourceDeleted && !targetDeleted) {
        keptEdges.push(edge);
        edgeKeys.add(edgeKey(edge.source, edge.sourceHandle, edge.target, edge.targetHandle));
        continue;
      }

      if (edgeKind(edge) !== "trigger") continue;

      if (!sourceDeleted && targetDeleted) {
        const incoming = incomingByDeletedTarget.get(edge.target);
        if (incoming) {
          incoming.push(edge);
        } else {
          incomingByDeletedTarget.set(edge.target, [edge]);
        }
      }

      if (sourceDeleted) {
        const outgoing = outgoingByDeletedSource.get(edge.source);
        if (outgoing) {
          outgoing.push(edge);
        } else {
          outgoingByDeletedSource.set(edge.source, [edge]);
        }
      }
    }

    const bypassEdges: Edge[] = [];
    const reachableOutputs = (deletedNodeId: string, visited = new Set<string>()): Edge[] => {
      if (visited.has(deletedNodeId)) return [];
      visited.add(deletedNodeId);

      const outputs: Edge[] = [];
      const outgoing = outgoingByDeletedSource.get(deletedNodeId) ?? [];
      for (const edge of outgoing) {
        if (deletedIds.has(edge.target)) {
          for (const output of reachableOutputs(edge.target, visited)) {
            outputs.push(output);
          }
        } else {
          outputs.push(edge);
        }
      }
      return outputs;
    };

    for (const [deletedNodeId, incomingEdges] of incomingByDeletedTarget) {
      const outgoingEdges = reachableOutputs(deletedNodeId);
      for (const incomingEdge of incomingEdges) {
        for (const outgoingEdge of outgoingEdges) {
          if (incomingEdge.source === outgoingEdge.target) continue;

          addReplacementEdge(
            bypassEdges,
            edgeKeys,
            incomingEdge.source,
            incomingEdge.sourceHandle ?? null,
            outgoingEdge.target,
            outgoingEdge.targetHandle ?? null,
          );
        }
      }
    }

    let nodeWriteIndex = 0;
    for (const node of nodes) {
      if (deletedIds.has(node.id)) continue;
      nodes[nodeWriteIndex] = node;
      nodeWriteIndex += 1;
    }
    nodes.length = nodeWriteIndex;

    commitNodes();
    for (const edge of bypassEdges) {
      keptEdges.push(edge);
    }
    edges = keptEdges;
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
    let unlistenMousePosition: (() => void) | undefined;

    void listenEvent<MousePositionPayload>("mouse_position", ({ payload }) => {
      scheduleMousePositionUpdate(payload);
      if (payload.clicked) {
        saveMouseClick(payload);
      }
    }).then((unlisten) => {
      unlistenMousePosition = unlisten;
    });

    void invoke<boolean>("is_mouse_position_monitoring")
      .then((active) => {
        isMousePositionMonitoring = active;
      })
      .catch(() => {
        isMousePositionMonitoring = false;
      });

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      unlistenMousePosition?.();
    };
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

  function pointerClientPosition(event: MouseEvent | TouchEvent) {
    if ("changedTouches" in event) {
      const touch = event.changedTouches[0];
      return touch ? { x: touch.clientX, y: touch.clientY } : undefined;
    }

    return { x: event.clientX, y: event.clientY };
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

    const styleValue =
      typeof node.style === "string"
        ? node.style.match(new RegExp(`${property}:\\s*(\\d+(?:\\.\\d+)?)px`))?.[1]
        : undefined;

    return Number(styleValue ?? (property === "width" ? 420 : 260));
  }

  function findSubflowGroupAt(position: { x: number; y: number }) {
    for (let index = nodes.length - 1; index >= 0; index -= 1) {
      const node = nodes[index];
      if (node.type !== "subflowGroupNode") continue;

      const width = nodeDimension(node, "width");
      const height = nodeDimension(node, "height");

      if (
        position.x >= node.position.x &&
        position.x <= node.position.x + width &&
        position.y >= node.position.y &&
        position.y <= node.position.y + height
      ) {
        return node;
      }
    }

    return undefined;
  }

  function createNodeAtPosition(
    type: string,
    nodeData: Record<string, unknown>,
    position: { x: number; y: number },
  ) {
    const parentSubflow = findSubflowGroupAt(position);

    return {
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
    const newNode = createNodeAtPosition(type, nodeData, position);

    nodes.push(newNode);
    commitNodes();
  }

  function filteredEdgeDropTemplates() {
    const query = edgeDropQuery.trim().toLowerCase();
    if (query === cachedEdgeDropQuery) return cachedEdgeDropTemplates;

    cachedEdgeDropQuery = query;
    if (!query) {
      cachedEdgeDropTemplates = nodeTemplates;
      return cachedEdgeDropTemplates;
    }

    cachedEdgeDropTemplates = [];
    for (const template of nodeTemplates) {
      const searchable = `${template.label} ${template.category}`.toLowerCase();
      if (searchable.includes(query)) cachedEdgeDropTemplates.push(template);
    }
    return cachedEdgeDropTemplates;
  }

  function closeEdgeDropMenu() {
    edgeDropMenu = undefined;
    edgeDropQuery = "";
  }

  function createNodeFromEdgeDrop(template: NodeTemplate) {
    if (!edgeDropMenu) return;

    const newNode = createNodeAtPosition(
      template.type,
      structuredClone(template.data),
      edgeDropMenu.flowPosition,
    );
    const selectedNewNode = { ...newNode, selected: true };

    clearNodeSelection(nodes);
    nodes.push(selectedNewNode);
    commitNodes();

    if (edgeDropMenu.sourceNodeId) {
      const newEdge = makeGraphEdge({
        source: edgeDropMenu.sourceNodeId,
        sourceHandle: edgeDropMenu.sourceHandle ?? FLOW_OUT_HANDLE,
        target: newNode.id,
        targetHandle: FLOW_IN_HANDLE,
      });

      clearEdgeSelection(edges);
      edges.push(newEdge);
      commitEdges();
    }

    selectedNodes.length = 0;
    selectedNodes.push(selectedNewNode);
    selectedNodes = selectedNodes;
    closeEdgeDropMenu();
  }

  function openNodeCreationMenuFromContextMenu() {
    if (!contextMenu) return;

    edgeDropQuery = "";
    edgeDropMenu = {
      x: contextMenu.x,
      y: contextMenu.y,
      flowPosition: contextMenu.flowPosition,
    };
    contextMenu = undefined;
    showRecordOptions = false;
    void tick().then(() => edgeDropSearchInput?.focus());
  }

  function handleSelectionChange(selection: { nodes: typeof nodes; edges: typeof edges }) {
    selectedNodes = selection.nodes;
  }

  function openContextMenu(event: MouseEvent, target: ContextMenuState["target"]) {
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
      canOptimizeTypingText: canOptimizeTypingTextSelection(),
    };
  }

  function handlePaneContextMenu({ event }: { event: MouseEvent }) {
    openContextMenu(event, "pane");
  }

  function handlePaneClick() {
    if (ignoreNextPaneClick) {
      ignoreNextPaneClick = false;
      return;
    }

    closeContextMenu();
  }

  function handleSelectionContextMenu({ event }: { event: MouseEvent; nodes: Node[] }) {
    openContextMenu(event, "selection");
  }

  function handleNodeContextMenu({ event, node }: { event: MouseEvent; node: Node }) {
    let nodeAlreadySelected = false;
    for (const selectedNode of selectedNodes) {
      if (selectedNode.id === node.id) {
        nodeAlreadySelected = true;
        break;
      }
    }

    if (!nodeAlreadySelected) {
      for (const candidate of nodes) {
        candidate.selected = candidate.id === node.id;
      }
      commitNodes();
      selectedNodes.length = 0;
      selectedNodes.push(node);
      selectedNodes = selectedNodes;
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

    submacros.push(result.submacro);
    submacros = submacros;
    nodes = result.nodes;
    edges = result.edges;
    selectedNodes.length = 0;
    selectedNodes = selectedNodes;
    closeContextMenu();
  }

  function addSubmacroInstance(definition: SubmacroDefinition) {
    const position = screenToFlowPosition({
      x: window.innerWidth / 2,
      y: window.innerHeight / 2,
    });

    nodes.push({
      id: crypto.randomUUID(),
      type: "submacroNode",
      position,
      data: submacroNodeData(definition),
    });
    commitNodes();
  }
</script>

<div class="flow-container" role="application" ondragover={onDragOver} ondrop={onDrop}>
  <SvelteFlow
    bind:nodes
    bind:edges
    {nodeTypes}
    {edgeTypes}
    {defaultEdgeOptions}
    {isValidConnection}
    onbeforeconnect={onBeforeConnect}
    onbeforedelete={onBeforeDelete}
    onconnectend={onConnectEnd}
    onselectionchange={handleSelectionChange}
    onpanecontextmenu={handlePaneContextMenu}
    onselectioncontextmenu={handleSelectionContextMenu}
    onnodecontextmenu={handleNodeContextMenu}
    onpaneclick={handlePaneClick}
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
      <button class="panel-btn" disabled={undoStack.length === 0} onclick={undo}> Undo </button>
      <button class="panel-btn" disabled={redoStack.length === 0} onclick={redo}> Redo </button>
      <div class="record-dropdown">
        <button
          class="panel-btn record-btn"
          class:recording={isRecording}
          disabled={isRunning}
          onclick={toggleRecording}
          aria-haspopup="menu"
          aria-expanded={showRecordOptions}
        >
          {isRecording ? "Stop Recording" : "Record"}
        </button>
        {#if showRecordOptions && !isRecording}
          <div class="record-menu" role="menu">
            <button role="menuitem" onclick={() => startRecordingWithMode("movesBeforeClicks")}>
              Clicks Only
            </button>
            <button role="menuitem" onclick={() => startRecordingWithMode("allMoves")}>
              All movement
            </button>
          </div>
        {/if}
      </div>
      <button class="panel-btn" disabled={isRecording} onclick={toggleExecution}>
        {isRunning ? "Stop" : "Run"}
      </button>
      <button class="panel-btn" onclick={() => (showVariables = !showVariables)}>
        Variables
      </button>
      <button class="panel-btn" onclick={() => (showMousePositionPanel = !showMousePositionPanel)}>
        Mouse
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
      {#if showMousePositionPanel}
        <div class="mouse-position-panel">
          <div class="mouse-position-header">
            <div>
              <div class="mouse-position-title">Mouse Position</div>
              <div class="mouse-position-status" class:active={isMousePositionMonitoring}>
                {isMousePositionMonitoring ? "Listening" : "Stopped"}
              </div>
            </div>
            <button class="panel-btn" onclick={toggleMousePositionMonitoring}>
              {isMousePositionMonitoring ? "Stop" : "Start"}
            </button>
          </div>
          <div class="mouse-coordinate">
            <span>X {currentMousePosition?.x ?? "--"}</span>
            <span>Y {currentMousePosition?.y ?? "--"}</span>
          </div>
          {#if mousePositionError}
            <div class="mouse-position-error">
              {mousePositionError}
            </div>
          {/if}
          <div class="mouse-click-header">
            <span>Clicks</span>
            <button onclick={clearSavedMouseClicks} disabled={savedMouseClicks.length === 0}>
              Clear
            </button>
          </div>
          <div class="mouse-click-list">
            {#key savedMouseClickVersion}
              {#each savedMouseClicks as click (click.id)}
                <div class="mouse-click-row">
                  <span>{click.button ?? "mouse"}</span>
                  <span>{click.x}, {click.y}</span>
                  <small>{click.createdAt}</small>
                </div>
              {:else}
                <div class="mouse-click-empty">No clicks saved.</div>
              {/each}
            {/key}
          </div>
        </div>
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
      <button role="menuitem" onclick={openNodeCreationMenuFromContextMenu}>New Node</button>
      <div class="menu-separator"></div>
      <button role="menuitem" disabled={undoStack.length === 0} onclick={undo}>Undo</button>
      <button role="menuitem" disabled={redoStack.length === 0} onclick={redo}>Redo</button>
      <div class="menu-separator"></div>
      <button role="menuitem" disabled={selectedNodes.length === 0} onclick={copySelection}>
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
        disabled={!contextMenu.canOptimizeTypingText}
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

  {#if edgeDropMenu}
    <div
      class="edge-drop-menu"
      style={`left: ${edgeDropMenu.x}px; top: ${edgeDropMenu.y}px;`}
      role="dialog"
      aria-label="Add node on edge drop"
      tabindex="-1"
      oncontextmenu={(event) => event.preventDefault()}
    >
      <input
        class="edge-drop-search"
        type="text"
        placeholder="Search components"
        bind:value={edgeDropQuery}
        bind:this={edgeDropSearchInput}
        onkeydown={(event) => {
          if (event.key === "Escape") {
            closeEdgeDropMenu();
          } else if (event.key === "Enter") {
            const [firstTemplate] = filteredEdgeDropTemplates();
            if (firstTemplate) createNodeFromEdgeDrop(firstTemplate);
          }
        }}
      />
      <div class="edge-drop-list">
        {#each filteredEdgeDropTemplates() as template (template.type)}
          <button onclick={() => createNodeFromEdgeDrop(template)}>
            <span>{template.label}</span>
            <small>{template.category}</small>
          </button>
        {:else}
          <div class="edge-drop-empty">No components found.</div>
        {/each}
      </div>
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
          <button class="panel-btn" onclick={() => createRecordedMacro("")}> Skip Keybind </button>
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

  .record-dropdown {
    display: inline-block;
    position: relative;
  }

  .record-menu {
    background: rgba(18, 18, 18, 0.98);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    box-shadow: 0 10px 24px rgba(0, 0, 0, 0.35);
    display: grid;
    gap: 0.2rem;
    min-width: 132px;
    padding: 0.25rem;
    position: absolute;
    right: 0;
    top: calc(100% + 0.35rem);
    z-index: 20;
  }

  .record-menu button {
    background: transparent;
    border: 0;
    border-radius: 4px;
    color: #e0e0e0;
    cursor: pointer;
    font: inherit;
    font-size: 12px;
    padding: 0.4rem 0.55rem;
    text-align: left;
  }

  .record-menu button:hover {
    background: #2c2d2f;
    color: #fff;
  }

  .panel-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
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

  .mouse-position-panel {
    background: rgba(18, 18, 18, 0.96);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    color: #e0e0e0;
    display: grid;
    gap: 0.55rem;
    min-width: 220px;
    padding: 0.65rem;
  }

  .mouse-position-header,
  .mouse-click-header,
  .mouse-coordinate,
  .mouse-click-row {
    align-items: center;
    display: flex;
    justify-content: space-between;
  }

  .mouse-position-header .panel-btn {
    margin-left: 0;
    padding: 4px 8px;
  }

  .mouse-position-title {
    color: #f1f1f1;
    font-size: 0.86rem;
    font-weight: 600;
  }

  .mouse-position-status {
    color: #888;
    font-size: 0.72rem;
  }

  .mouse-position-status.active {
    color: #8fd4ff;
  }

  .mouse-coordinate {
    background: #232426;
    border: 1px solid #414141;
    border-radius: 4px;
    color: #f1f1f1;
    font-family: "Fira Mono", monospace;
    font-size: 0.82rem;
    gap: 0.75rem;
    padding: 0.45rem 0.55rem;
  }

  .mouse-position-error {
    color: #ff8aa8;
    font-size: 0.74rem;
  }

  .mouse-click-header {
    color: #888;
    font-size: 0.72rem;
    text-transform: uppercase;
  }

  .mouse-click-header button {
    background: transparent;
    border: 0;
    color: #8fd4ff;
    cursor: pointer;
    font: inherit;
    padding: 0;
    text-transform: none;
  }

  .mouse-click-header button:disabled {
    color: #555;
    cursor: not-allowed;
  }

  .mouse-click-list {
    display: grid;
    gap: 0.25rem;
    max-height: 160px;
    overflow: auto;
    scrollbar-color: #4a4a4a #171717;
    scrollbar-width: thin;
  }

  .mouse-click-row {
    background: #232426;
    border: 1px solid #353535;
    border-radius: 4px;
    gap: 0.5rem;
    padding: 0.35rem 0.45rem;
  }

  .mouse-click-row span {
    color: #f1f1f1;
    font-size: 0.76rem;
  }

  .mouse-click-row small,
  .mouse-click-empty {
    color: #888;
    font-size: 0.7rem;
  }

  .mouse-click-empty {
    padding: 0.35rem 0.1rem;
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

  .edge-drop-menu {
    background: rgba(18, 18, 18, 0.98);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    box-shadow: 0 14px 32px rgba(0, 0, 0, 0.36);
    display: grid;
    gap: 0.35rem;
    max-height: min(360px, calc(100vh - 2rem));
    min-width: 220px;
    padding: 0.4rem;
    position: fixed;
    transform: translate(-0.35rem, -0.35rem);
    z-index: 25;
  }

  .edge-drop-search {
    background: #232426;
    border: 1px solid #414141;
    border-radius: 4px;
    color: #f1f1f1;
    font: inherit;
    font-size: 0.8rem;
    outline: none;
    padding: 0.42rem 0.5rem;
  }

  .edge-drop-search:focus {
    border-color: #2a8af6;
    box-shadow: 0 0 0 1px rgba(42, 138, 246, 0.35);
  }

  .edge-drop-list {
    display: grid;
    gap: 0.15rem;
    max-height: 300px;
    overflow: auto;
    padding-right: 0.1rem;
    scrollbar-color: #4a4a4a #171717;
    scrollbar-width: thin;
  }

  .edge-drop-list::-webkit-scrollbar {
    width: 8px;
  }

  .edge-drop-list::-webkit-scrollbar-track {
    background: #171717;
  }

  .edge-drop-list::-webkit-scrollbar-thumb {
    background: #3a3b3d;
    border: 2px solid #171717;
    border-radius: 999px;
  }

  .edge-drop-list button {
    align-items: center;
    background: transparent;
    border: 0;
    border-radius: 4px;
    color: #f1f1f1;
    cursor: pointer;
    display: flex;
    font: inherit;
    gap: 0.75rem;
    justify-content: space-between;
    padding: 0.42rem 0.5rem;
    text-align: left;
  }

  .edge-drop-list button:hover {
    background: #2c2d2f;
  }

  .edge-drop-list small,
  .edge-drop-empty {
    color: #888;
    font-size: 0.72rem;
  }

  .edge-drop-empty {
    padding: 0.45rem 0.5rem;
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
