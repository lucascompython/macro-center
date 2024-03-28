<script context="module" lang="ts">
  export type BaseNodeProps = NodeProps & {
    data: {
      title: string;
      subline?: string;
      enableTarget?: boolean;
      enableSource?: boolean;
    };
  };
</script>

<script lang="ts">
  import {
    Handle,
    Position,
    type NodeProps,
    useConnection,
  } from "@xyflow/svelte";
  import SettingsIcon from "@assets/SettingsIcon.svelte";

  export let data: BaseNodeProps["data"];

  export let id: BaseNodeProps["id"];

  // #region Unused props that are here just for not having the "unknown prop" warning

  export let dragHandle: BaseNodeProps["dragHandle"] = undefined;
  dragHandle;
  export let type: BaseNodeProps["type"] = undefined;
  type;
  export let selected: BaseNodeProps["selected"] = undefined;
  selected;
  export let isConnectable: BaseNodeProps["isConnectable"];
  isConnectable;
  export let zIndex: BaseNodeProps["zIndex"];
  zIndex;
  export let width: BaseNodeProps["width"] = undefined;
  width;
  export let height: BaseNodeProps["height"] = undefined;
  height;
  export let dragging: BaseNodeProps["dragging"];
  dragging;
  export let targetPosition: BaseNodeProps["targetPosition"] = undefined;
  targetPosition;
  export let sourcePosition: BaseNodeProps["sourcePosition"] = undefined;
  sourcePosition;
  export let positionAbsoluteX: BaseNodeProps["positionAbsoluteX"];
  positionAbsoluteX;
  export let positionAbsoluteY: BaseNodeProps["positionAbsoluteY"];
  positionAbsoluteY;

  // #endregion

  const connection = useConnection();

  let isConnecting = false;
  let isTarget = false;

  $: {
    data.enableTarget = data.enableTarget ?? true;
    data.enableSource = data.enableSource ?? true;

    isConnecting = !!$connection.startHandle?.nodeId;
    isTarget =
      !!$connection.startHandle &&
      $connection.startHandle?.nodeId !== id &&
      $connection.startHandle?.type === "source";
  }
</script>

<div class="wrapper gradient">
  <div class="inner">
    <div class="top-part">
      <div class="title">
        {data.title}
        {#if data.subline}
          <div class="subline">{data.subline}</div>
        {/if}
      </div>

      <div class="settings-icon">
        <button
          on:click={(e) => {
            console.log("ola");
            console.log("ola2");
          }}
        >
          <SettingsIcon />
        </button>
      </div>
    </div>
    <div class="content">
      <slot />
    </div>
  </div>
  {#if data.enableTarget}
    <Handle
      style="border-color: {isTarget ? '#e92a67' : ''}"
      type="target"
      position={Position.Left}
    />
  {/if}
  {#if data.enableSource}
    <Handle
      style="border-color: {isConnecting &&
      !isTarget &&
      id !== $connection.startHandle?.nodeId
        ? '#2a8af6'
        : ''}"
      type="source"
      position={Position.Right}
    />
  {/if}
</div>

<style>
  .top-part {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .settings-icon {
    margin-left: 6rem;
  }

  .settings-icon button {
    background: none;
    border: none;
    cursor: pointer;
  }
  .subline {
    font-size: 12px;
    color: #777;
    margin-top: 0.35rem;
    margin-bottom: 0.5rem;
  }

  .content {
    background: #2c2d2f;
    width: 100%;
    min-height: 1.5rem;
    cursor: default;
    border-radius: 0.25rem;
  }
</style>
