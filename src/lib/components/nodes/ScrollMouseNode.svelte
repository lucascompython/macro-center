<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import { valueHandle } from "$lib/graph";
  import { Axis, type ScrollMouseNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: ScrollMouseNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();

  function handleAxisChange(e: Event & { currentTarget: HTMLSelectElement }) {
    updateNodeData(id, { axis: e.currentTarget.value as Axis });
  }

  function handleAmountInput(value: string) {
    updateNodeData(id, { amount: Number(value) || 0 });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Scroll Mouse",
    subline: data.subline ?? "Scroll the mouse wheel",
  }}
>
  <div class="input-container">
    <span>Axis:</span>
    <select class="nodrag" value={data.axis ?? Axis.VERTICAL} onchange={handleAxisChange}>
      <option value={Axis.VERTICAL}>Vertical</option>
      <option value={Axis.HORIZONTAL}>Horizontal</option>
    </select>
  </div>

  <div class="input-container">
    <span>Amount:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="amount"
      inputmode="numeric"
      value={data.amount ?? 3}
      onInput={handleAmountInput}
    />
  </div>
</BaseNode>

<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("amount")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
