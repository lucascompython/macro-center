<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import { valueHandle } from "$lib/graph";
  import { CoordinateMode, type MouseMoveNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: MouseMoveNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();

  function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
    updateNodeData(id, { coordinateMode: e.currentTarget.value as CoordinateMode });
  }

  function handleCoordinateInput(field: "x" | "y", value: string) {
    updateNodeData(id, { [field]: Number(value) || 0 });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Move Mouse",
    subline: data.subline ?? "Move cursor to position",
  }}
>
  <div class="input-container">
    <span>X:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="x"
      inputmode="numeric"
      value={data.x ?? 0}
      onInput={(value) => handleCoordinateInput("x", value)}
    />
  </div>

  <div class="input-container">
    <span>Y:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="y"
      inputmode="numeric"
      value={data.y ?? 0}
      onInput={(value) => handleCoordinateInput("y", value)}
    />
  </div>

  <div class="input-container">
    <span>Mode:</span>
    <select
      class="nodrag"
      value={data.coordinateMode ?? CoordinateMode.ABSOLUTE}
      onchange={handleModeChange}
    >
      <option value={CoordinateMode.ABSOLUTE}>Absolute</option>
      <option value={CoordinateMode.RELATIVE}>Relative</option>
    </select>
  </div>
</BaseNode>

<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("x")}
  style="left: 35%; border-color: #38d0ff; background: #102a36"
/>
<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("y")}
  style="left: 65%; border-color: #38d0ff; background: #102a36"
/>
