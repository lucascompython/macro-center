<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import { valueHandle } from "$lib/graph";
  import type { TypeNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: TypeNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();

  function handleTextChange(value: string) {
    updateNodeData(id, { text: value });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Type Text",
    subline: data.subline ?? "Types a given text",
  }}
>
  <div class="input-container">
    <span>Text:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="text"
      value={data.text ?? "Hello World!"}
      onInput={handleTextChange}
    />
  </div>
</BaseNode>

<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("text")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
