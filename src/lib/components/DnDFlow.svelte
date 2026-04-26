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
  import GradientEdge from "./nodes/GradientEdge.svelte";

  import { initialNodes, initialEdges } from "$lib/initial-nodes";

  import { save, open } from "@tauri-apps/plugin-dialog";
  import {
    writeTextFile,
    readTextFile,
    BaseDirectory,
  } from "@tauri-apps/plugin-fs";

  // use $state.raw for performance as recommended by xyflow docs
  let nodes = $state.raw(initialNodes);
  let edges = $state.raw(initialEdges);

  const nodeTypes: NodeTypes = {
    keyBindNode: KeyBindNode,
    typeNode: TypeNode,
    keyNode: KeyNode,
    mousePressNode: MousePressNode,
    mouseMoveNode: MouseMoveNode,
    delayNode: DelayNode,
    scrollMouseNode: ScrollMouseNode,
    conditionalNode: ConditionalNode,
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
    return true;
  };

  // DnD Hook
  const { screenToFlowPosition, toObject, setViewport } = useSvelteFlow();

  import { MacroRunner } from "$lib/runner/MacroRunner";
  import { onDestroy } from "svelte";

  let runner: MacroRunner | undefined = $state();
  let isRunning = $state(false);

  function toggleExecution() {
    if (isRunning) {
      runner?.cleanup();
      isRunning = false;
    } else {
      if (!runner) {
        runner = new MacroRunner(nodes, edges);
      }
      runner.updateGraph(nodes, edges);
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
        await writeTextFile(filePath, JSON.stringify(flowData, null));
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

    const newNode = {
      id: crypto.randomUUID(),
      type,
      position,
      data: nodeData,
      origin: [0.5, 0.5] as [number, number],
    };

    nodes = [...nodes, newNode];
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
      <button class="panel-btn" onclick={saveMacro}>Save</button>
      <button class="panel-btn" onclick={loadMacro}>Load</button>
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
</style>
