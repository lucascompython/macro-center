<script lang="ts">
  import { writable } from "svelte/store";
  import {
    SvelteFlow,
    Background,
    BackgroundVariant,
    MiniMap,
    type Node as NodeType,
    type Edge as EdgeType,
    type IsValidConnection,
    Controls,
    type EdgeTypes,
    type NodeTypes,
  } from "@xyflow/svelte";

  import BaseNode from "./Nodes/BaseNode/BaseNode.svelte";
  import Edge from "./Nodes/BaseNode/Edge.svelte";
  import "@xyflow/svelte/dist/style.css";
  import "./Nodes/BaseNode/style.css";

  import TypeNode from "./Nodes/TypeNode.svelte";
  import KeyBindNode from "./Nodes/KeyBindNode.svelte";
  import KeyPressNode from "./Nodes/KeyPressNode.svelte";

  import { initialNodes, initialEdges } from "./nodes-and-edges";

  const nodes = writable<NodeType[]>(initialNodes);
  const edges = writable<EdgeType[]>(initialEdges);

  const nodeTypes: NodeTypes = {
    // colorPicker: TypeNode,
    // baseNode: BaseNode,
    base: BaseNode,
    typeNode: TypeNode,
    keyBindNode: KeyBindNode,
    keyPressNode: KeyPressNode,
  };

  const edgeTypes: EdgeTypes = {
    turbo: Edge,
  };

  const defaultEdgeOptions = {
    type: "turbo",
    markEnd: "edge-circle",
  };

  const isValidConnection: IsValidConnection = (connection) => {
    if (connection.source === connection.target) {
      return false;
    }
    return true;
  };
</script>

<div class="flow">
  <SvelteFlow
    {nodes}
    {edges}
    {nodeTypes}
    {edgeTypes}
    {defaultEdgeOptions}
    {isValidConnection}
    fitView
    proOptions={{ hideAttribution: true }}
    zoomOnDoubleClick={false}
    colorMode="dark"
    on:nodeclick={(event) => console.log("on node click", event.detail.node)}
  >
    <Controls />
    <svg>
      <defs>
        <linearGradient id="edge-gradient">
          <stop offset="0%" stop-color="#e92a67" />
          <stop offset="25%" stop-color="#ae53ba" />
          <stop offset="75%" stop-color="#2a8af6" />
          <stop offset="100%" stop-color="#e92a67" />
        </linearGradient>
        <marker
          id="edge-circle"
          viewBox="-5 -5 10 10"
          refX="0"
          refY="0"
          markerUnits="strokeWidth"
          markerWidth="10"
          markerHeight="10"
          orient="auto"
        >
          <circle stroke="#2a8af6" stroke-opacity="0.75" r="2" cx="0" cy="0" />
        </marker>
      </defs>
    </svg>
    <Background
      variant={BackgroundVariant.Lines}
      bgColor="#121212"
      patternColor="#242424"
      gap={25}
    />
    <MiniMap />
  </SvelteFlow>
</div>

<style>
  .flow {
    top: 0;
    left: 0;
    height: 100vh;
    width: 100%;
    position: absolute;
  }
</style>
