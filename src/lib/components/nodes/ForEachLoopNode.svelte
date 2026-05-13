<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import { parseLiteral, valueToInput } from "$lib/logic";
  import { valueHandle } from "$lib/graph";
  import type { ForEachLoopNodeData, MacroValue } from "$lib/types";

  interface Props {
    id: string;
    data: ForEachLoopNodeData;
  }

  let { id, data }: Props = $props();
  const { updateNodeData } = useSvelteFlow();

  function handleItemsInput(value: string) {
    const parsed = parseLiteral(value);
    updateNodeData(id, {
      items: Array.isArray(parsed) ? (parsed as MacroValue[]) : [],
    });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "For Each",
    subline: data.subline ?? "Run once per list item",
    enableSource: false,
  }}
>
  <div class="input-container">
    <span>Items:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="items"
      value={valueToInput(data.items ?? [])}
      placeholder="['a','b']"
      onChange={handleItemsInput}
    />
  </div>
  <div class="input-container">
    <span>Item:</span>
    <input
      class="nodrag"
      type="text"
      value={data.itemVariable ?? "item"}
      oninput={(event) => updateNodeData(id, { itemVariable: event.currentTarget.value })}
    />
  </div>
  <div class="input-container">
    <span>Index:</span>
    <input
      class="nodrag"
      type="text"
      value={data.indexVariable ?? "index"}
      oninput={(event) => updateNodeData(id, { indexVariable: event.currentTarget.value })}
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
  id={valueHandle("items")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
<Handle type="source" position={Position.Right} id="body" style="top: 66%; border-color: #ffc857" />
<Handle type="source" position={Position.Right} id="done" style="top: 86%; border-color: #27d209" />

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
