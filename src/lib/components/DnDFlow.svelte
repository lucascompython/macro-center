<script lang="ts">
    import {
        SvelteFlow,
        Background,
        BackgroundVariant,
        MiniMap,
        Controls,
        useSvelteFlow,
        type NodeTypes,
        type EdgeTypes,
        type IsValidConnection,
    } from "@xyflow/svelte";

    import "@xyflow/svelte/dist/style.css";
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
    const { screenToFlowPosition } = useSvelteFlow();

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
    .flow-container {
        position: absolute;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
    }
</style>
