<script lang="ts">
  import BaseNode, { type BaseNodeProps } from "./BaseNode/BaseNode.svelte";

  import "./style.css";

  type $$Props = BaseNodeProps & {
    data: {
      keyBind: string;
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
    title: "KeyBind Node",
    subline: "Subline",
    keyBind: "F6",
  };
</script>

<BaseNode
  {id}
  data={{
    title: data.title,
    subline: data.subline,
    enableTarget: false,
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
    Key Bind:
    <span class="nodrag" contenteditable="true" bind:innerText={data.keyBind}
    ></span>
  </div>
</BaseNode>

<style>
  span {
    max-width: 130px;
  }
</style>
