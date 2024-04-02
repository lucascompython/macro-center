<script lang="ts">
  import BaseNode, {
    type BaseNodeProps,
  } from "@components/Nodes/BaseNode/BaseNode.svelte";

  import "./node-style.css";

  import NumberOnlySpan from "@components/NumberOnlySpan.svelte";

  import { Axis } from "@utils";

  type $$Props = BaseNodeProps & {
    data: {
      axis: Axis;
      length: number;
    };
  };

  // #region Unused props that are here just for not having the "unknown prop" warning

  export let id: $$Props["id"];
  id;
  export let dragHandle: $$Props["dragHandle"] = undefined;
  dragHandle;
  export let type: $$Props["type"] = undefined;
  type;
  export let selected: $$Props["selected"] = undefined;
  selected;
  export let isConnectable: $$Props["isConnectable"];
  isConnectable;
  export let zIndex: $$Props["zIndex"];
  zIndex;
  export let width: $$Props["width"] = undefined;
  width;
  export let height: $$Props["height"] = undefined;
  height;
  export let dragging: $$Props["dragging"];
  dragging;
  export let targetPosition: $$Props["targetPosition"] = undefined;
  targetPosition;
  export let sourcePosition: $$Props["sourcePosition"] = undefined;
  sourcePosition;
  export let positionAbsoluteX: $$Props["positionAbsoluteX"];
  positionAbsoluteX;
  export let positionAbsoluteY: $$Props["positionAbsoluteY"];
  positionAbsoluteY;

  // #endregion

  export let data: $$Props["data"] = {
    title: "Scroll Node",
    subline: "Subline",
    axis: Axis.VERTICAL,
    length: 100,
  };
</script>

<BaseNode
  {id}
  data={{
    title: data.title,
    subline: data.subline,
  }}
  {dragHandle}
  {type}
  {selected}
  {isConnectable}
  {zIndex}
  {width}
  {height}
  {dragging}
  {targetPosition}
  {sourcePosition}
  {positionAbsoluteX}
  {positionAbsoluteY}
>
  <div class="input-container">
    Axis: <select class="nodrag" bind:value={data.mode}>
      <option value={Axis.VERTICAL}>Vertical</option>
      <option value={Axis.HORIZONTAL}>Horizontal</option>
    </select>
  </div>

  <div class="input-container">
    Length: <NumberOnlySpan intOnly={true} bind:value={data.length} />
  </div>
</BaseNode>
