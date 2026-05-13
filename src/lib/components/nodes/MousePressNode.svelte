<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedSelect from "$lib/components/ValueBackedSelect.svelte";
  import { valueHandle } from "$lib/graph";
  import { ActionMode, MouseButton, type MousePressNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: MousePressNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();
  const buttonOptions: { value: string; label: string }[] = [
    { value: MouseButton.LEFT, label: "Left" },
    { value: MouseButton.RIGHT, label: "Right" },
    { value: MouseButton.MIDDLE, label: "Middle" },
    { value: MouseButton.MOUSE4, label: "Mouse4" },
    { value: MouseButton.MOUSE5, label: "Mouse5" },
  ];

  function handleButtonChange(value: string) {
    updateNodeData(id, { button: value as MouseButton });
  }

  function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
    updateNodeData(id, { mode: e.currentTarget.value as ActionMode });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Mouse Press",
    subline: data.subline ?? "Simulate mouse button action",
  }}
>
  <div class="input-container">
    <span>Button:</span>
    <ValueBackedSelect
      nodeId={id}
      inputName="button"
      value={data.button ?? MouseButton.LEFT}
      options={buttonOptions}
      onChange={handleButtonChange}
    />
  </div>

  <div class="input-container">
    <span>Mode:</span>
    <select class="nodrag" value={data.mode ?? ActionMode.CLICK} onchange={handleModeChange}>
      <option value={ActionMode.CLICK}>Click</option>
      <option value={ActionMode.PRESS}>Press</option>
      <option value={ActionMode.RELEASE}>Release</option>
    </select>
  </div>
</BaseNode>

<Handle
  type="target"
  position={Position.Top}
  id={valueHandle("button")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
