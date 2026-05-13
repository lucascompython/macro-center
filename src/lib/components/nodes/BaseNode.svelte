<script lang="ts">
  import { Handle, Position, useConnection } from "@xyflow/svelte";
  import SettingsIcon from "$lib/components/icons/SettingsIcon.svelte";
  import type { Snippet } from "svelte";
  import type { BaseNodeData } from "$lib/types";
  import { FLOW_IN_HANDLE, FLOW_OUT_HANDLE } from "$lib/graph";

  interface Props {
    id: string;
    data: BaseNodeData & {
      enableTarget?: boolean;
      enableSource?: boolean;
    };
    children?: Snippet;
  }

  let { id, data, children }: Props = $props();

  const connection = useConnection();

  // Default enable both handles
  const enableTarget = $derived(data.enableTarget ?? true);
  const enableSource = $derived(data.enableSource ?? true);

  // Connection state - simplified to avoid TypeScript issues with union type
  // Just check if we're in a connection state
  const isConnecting = $derived(connection.current.inProgress);
  const connectionState = $derived(
    connection.current as {
      inProgress: boolean;
      startHandle?: { nodeId?: string; type?: string };
    },
  );

  const isTarget = $derived(
    connectionState.inProgress &&
      connectionState.startHandle?.nodeId !== id &&
      connectionState.startHandle?.type === "source",
  );

  const showSourceHighlight = $derived(
    isConnecting && !isTarget && connectionState.startHandle?.nodeId !== id,
  );

  function handleSettingsClick() {
    console.log("Settings clicked for node:", id);
    // TODO: Open node settings modal
  }
</script>

<div class="base-node-container">
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
          <button onclick={handleSettingsClick}>
            <SettingsIcon />
          </button>
        </div>
      </div>
      <div class="content">
        {#if children}
          {@render children()}
        {/if}
      </div>
    </div>
  </div>
  {#if enableTarget}
    <Handle
      type="target"
      position={Position.Left}
      id={FLOW_IN_HANDLE}
      style="border-color: {isTarget ? '#e92a67' : ''}"
    />
  {/if}
  {#if enableSource}
    <Handle
      type="source"
      position={Position.Right}
      id={FLOW_OUT_HANDLE}
      style="border-color: {showSourceHighlight ? '#2a8af6' : ''}"
    />
  {/if}
</div>

<style>
  .base-node-container {
    position: relative;
  }

  .top-part {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .settings-icon {
    margin-left: 3rem;
  }

  .settings-icon button {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
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
    min-height: 0.5rem;
    cursor: default;
    border-radius: 0.25rem;
    padding: 0.25rem 0;
  }

  .content:empty {
    display: none;
  }
</style>
