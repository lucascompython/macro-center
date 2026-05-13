<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ConditionEditor from "$lib/components/ConditionEditor.svelte";
  import { defaultConditionExpression } from "$lib/logic";
  import { valueHandle } from "$lib/graph";
  import type { ConditionExpression, WhileLoopNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: WhileLoopNodeData;
  }

  let { id, data }: Props = $props();
  const { updateNodeData } = useSvelteFlow();

  function handleConditionChange(conditionExpression: ConditionExpression) {
    updateNodeData(id, { conditionExpression });
  }

  function handleMaxIterationsInput(event: Event & { currentTarget: HTMLInputElement }) {
    updateNodeData(id, { maxIterations: Number(event.currentTarget.value) || 0 });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "While",
    subline: data.subline ?? "Run while true",
    enableSource: false,
  }}
>
  <ConditionEditor
    nodeId={id}
    expression={data.conditionExpression ?? defaultConditionExpression}
    onChange={handleConditionChange}
  />
  <div class="input-container">
    <span>Index:</span>
    <input
      class="nodrag"
      type="text"
      value={data.indexVariable ?? "index"}
      oninput={(event) => updateNodeData(id, { indexVariable: event.currentTarget.value })}
    />
  </div>
  <div class="input-container">
    <span>Max:</span>
    <input
      class="nodrag"
      type="text"
      inputmode="numeric"
      value={data.maxIterations ?? 100}
      oninput={handleMaxIterationsInput}
    />
  </div>
  <div class="loop-outputs">
    <span>Body</span>
    <span>Done</span>
  </div>
</BaseNode>

<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("condition")}
  style="left: 25%; border-color: #38d0ff; background: #102a36"
/>
<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("left")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("right")}
  style="left: 75%; border-color: #38d0ff; background: #102a36"
/>
<Handle type="source" position={Position.Right} id="body" style="top: 64%; border-color: #ffc857" />
<Handle type="source" position={Position.Right} id="done" style="top: 84%; border-color: #27d209" />

<style>
  .loop-outputs {
    color: #d8d8d8;
    display: flex;
    flex-direction: column;
    font-size: 0.75rem;
    gap: 0.5rem;
    margin: 0.45rem 0.5rem 0;
    text-align: right;
  }
</style>
