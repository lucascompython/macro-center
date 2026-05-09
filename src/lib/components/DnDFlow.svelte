<script lang="ts">
  import {
    SvelteFlow,
    Background,
    BackgroundVariant,
    MiniMap,
    Controls,
    Panel,
    useSvelteFlow,
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
  import { onDestroy, setContext } from "svelte";

  // use $state.raw for performance as recommended by xyflow docs
  let nodes = $state.raw(initialNodes);
  let edges = $state.raw(initialEdges);
  let variables = $state.raw([] as VariableDefinition[]);
  let submacros = $state.raw([] as SubmacroDefinition[]);
  let selectedNodes = $state.raw([] as typeof nodes);
  let variableSnapshot = $state({} as Record<string, MacroValue>);
  let showVariables = $state(false);

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
    if (runner) {
      runner.cleanup();
    }
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
</style>
