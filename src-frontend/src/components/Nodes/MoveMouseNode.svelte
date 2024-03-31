<script lang="ts">
  import BaseNode, { type BaseNodeProps } from "./BaseNode/BaseNode.svelte";

  import "./node-style.css";

  import NumberOnlySpan from "@components/NumberOnlySpan.svelte";

  import { CoordinateMode } from "@utils";

  type $$Props = BaseNodeProps & {
    data: {
      x: number;
      y: number;
      coordinateMode: CoordinateMode;
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
    title: "Move Mouse",
    subline: "Moves the mouse to a specific position",
    x: 0,
    y: 0,
    coordinateMode: CoordinateMode.RELATIVE,
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
    X: <NumberOnlySpan intOnly={true} bind:value={data.x} />
  </div>

  <div class="input-container">
    Y: <NumberOnlySpan intOnly={true} bind:value={data.y} />
  </div>

  <div class="input-container">
    Coordinate mode:
    <select class="nodrag" bind:value={data.coordinateMode}>
      <option value={CoordinateMode.RELATIVE}>Relative</option>
      <option value={CoordinateMode.ABSOLUTE}>Absolute</option>
    </select>
  </div>
</BaseNode>
