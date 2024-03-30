<script lang="ts">
  import BaseNode, {
    type BaseNodeProps,
  } from "@components/Nodes/BaseNode/BaseNode.svelte";
  import "./node-style.css";

  import NumberOnlySpan from "@components/NumberOnlySpan.svelte";

  type $$Props = BaseNodeProps & {
    data: {
      text: string;
      delay: number;
      cancelKey?: string;
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
    text: "Type something",
    delay: 69,
    title: "Type Node",
    subline: "Subline",
    cancelKey: "",
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
    Text:
    <span class="nodrag" contenteditable="true" bind:innerText={data.text}
    ></span>
  </div>

  <div class="input-container">
    Delay(s):
    <NumberOnlySpan maxWidth="140" bind:value={data.delay} />
  </div>

  <div class="input-container">
    Cancel key:
    <span
      class="nodrag"
      class:optional-placeholder={!data.cancelKey}
      contenteditable="true"
      role="textbox"
      bind:textContent={data.cancelKey}
    ></span>
  </div>
</BaseNode>
