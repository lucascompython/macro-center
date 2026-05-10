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

  import { save, open } from "@tauri-apps/plugin-dialog";
  import {
    writeTextFile,
    readTextFile,
  } from "@tauri-apps/plugin-fs";
  import { MacroRunner } from "$lib/runner/MacroRunner";
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

  onDestroy(() => {
    if (historyTimer) {
      clearTimeout(historyTimer);
    }

    if (runner) {
      runner.cleanup();
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
      <button class="panel-btn" onclick={toggleExecution}>
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
        class="danger"
        disabled={selectedNodes.length === 0}
        onclick={deleteSelection}
      >
        Delete
      </button>
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

  .panel-btn:disabled {
    cursor: not-allowed;
    opacity: 0.45;
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

  .menu-separator {
    background: #303030;
    height: 1px;
    margin: 0.15rem 0;
  }
</style>
