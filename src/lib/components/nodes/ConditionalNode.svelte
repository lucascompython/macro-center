<script lang="ts">
  import { Handle, Position, useSvelteFlow, useConnection } from "@xyflow/svelte";
  import SettingsIcon from "$lib/components/icons/SettingsIcon.svelte";
  import ConditionEditor from "$lib/components/ConditionEditor.svelte";
  import { defaultConditionExpression } from "$lib/logic";
  import { FLOW_IN_HANDLE, valueHandle } from "$lib/graph";
  import type { ConditionalNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: ConditionalNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();
  const connection = useConnection();

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

  function handleConditionChange(
    conditionExpression: NonNullable<ConditionalNodeData["conditionExpression"]>,
  ) {
    updateNodeData(id, { conditionExpression });
  }

  function handleSettingsClick() {
    console.log("Settings clicked for node:", id);
  }
</script>

<div class="base-node-container">
  <div class="wrapper gradient">
    <div class="inner">
      <div class="top-part">
        <div class="title">
          {data.title ?? "Conditional"}
          <div class="subline">{data.subline ?? "Branch based on condition"}</div>
        </div>

        <div class="settings-icon">
          <button onclick={handleSettingsClick}>
            <SettingsIcon />
          </button>
        </div>
      </div>
      <div class="content">
        <ConditionEditor
          nodeId={id}
          expression={data.conditionExpression ?? defaultConditionExpression}
          onChange={handleConditionChange}
        />

        <div class="outputs">
          <div class="output-label true-label">True</div>
          <div class="output-label false-label">False</div>
        </div>
      </div>
    </div>
  </div>

  <Handle
    type="target"
    position={Position.Left}
    id={FLOW_IN_HANDLE}
    style="border-color: {isTarget ? '#e92a67' : ''}"
  />

  <Handle
    type="target"
    position={Position.Top}
    id={valueHandle("left")}
    style="left: 34%; border-color: #38d0ff; background: #102a36"
  />

  <Handle
    type="target"
    position={Position.Top}
    id={valueHandle("right")}
    style="left: 66%; border-color: #38d0ff; background: #102a36"
  />

  <Handle
    type="source"
    position={Position.Right}
    id="true"
    style="top: 40%; border-color: #27d209"
  />

  <Handle
    type="source"
    position={Position.Right}
    id="false"
    style="top: 70%; border-color: #ff6060"
  />
</div>

<style>
  .base-node-container {
    position: relative;
  }

  .top-part {
    align-items: center;
    display: flex;
    justify-content: space-between;
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
    color: #777;
    font-size: 12px;
    margin-bottom: 0.5rem;
    margin-top: 0.35rem;
  }

  .content {
    background: #2c2d2f;
    border-radius: 0.25rem;
    cursor: default;
    min-height: 0.5rem;
    padding: 0.25rem 0;
    width: 100%;
  }

  .outputs {
    align-items: flex-end;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: 0.5rem;
    padding-right: 0.5rem;
  }

  .output-label {
    border-radius: 0.25rem;
    font-size: 0.75rem;
    padding: 0.1rem 0.3rem;
  }

  .true-label {
    color: #27d209;
  }

  .false-label {
    color: #ff6060;
  }
</style>
