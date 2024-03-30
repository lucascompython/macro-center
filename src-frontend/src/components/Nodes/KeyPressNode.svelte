<script lang="ts">
  import BaseNode, {
    type BaseNodeProps,
  } from "@components/Nodes/BaseNode/BaseNode.svelte";

  import "./node-style.css";

  import { KeyPressMode } from "@utils";

  import NumberOnlySpan from "@components/NumberOnlySpan.svelte";

  type $$Props = BaseNodeProps & {
    data: {
      key: string;
      numTimes: number;
      mode: KeyPressMode;
      delay?: number;
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
    title: "Key Press Node",
    subline: "Subline",
    key: "F6",
    numTimes: 1,
    mode: KeyPressMode.HOLD,
    cancelKey: "esc",
    delay: 0.4,
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
    Key: <span class="nodrag" contenteditable="true" bind:innerText={data.key}
    ></span>
  </div>

  <div class="input-container">
    Number of times:

    <NumberOnlySpan bind:value={data.numTimes} intOnly={true} />
  </div>

  <div class="input-container">
    Mode:
    <select class="nodrag" bind:value={data.mode}>
      <option value={KeyPressMode.PRESS}>Press</option>
      <option value={KeyPressMode.HOLD}>Hold</option>
      <option value={KeyPressMode.RELEASE}>Release</option>
    </select>
  </div>

  <div class="input-container">
    Delay:
    <NumberOnlySpan bind:value={data.delay} />
  </div>

  <div class="input-container">
    Cancel key:
    <span
      class="nodrag"
      contenteditable="true"
      role="textbox"
      class:optional-placeholder={!data.cancelKey}
      bind:textContent={data.cancelKey}
    ></span>
  </div>
</BaseNode>

<style>
  select {
    -webkit-appearance: none; /* Remove the fucking ugly ass default webkit select box */
    appearance: none;
    padding: 0.1rem;
    border-radius: 0.25rem;
    border: 1px solid transparent;
    margin-left: 0.5rem;
    background-color: #4c4d4f;
    color: white;
    background-image: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" fill="white" viewBox="0 0 16 16"><path fill-rule="evenodd" d="M1.646 4.646a.5.5 0 0 1 .708 0L8 10.293l5.646-5.647a.5.5 0 0 1 .708.708l-6 6a.5.5 0 0 1-.708 0l-6-6a.5.5 0 0 1 0-.708"/></svg>');
    background-repeat: no-repeat;
    background-position: right 0.5rem center;
    background-position-y: 0.2rem;
    padding-right: 2rem;
    transition: border 0.2s ease;
  }

  select:focus {
    border: 1px solid #e92a67;
  }
</style>
