<script lang="ts">
  import { Handle, Position, useSvelteFlow } from "@xyflow/svelte";
  import BaseNode from "./BaseNode.svelte";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import { valueHandle } from "$lib/graph";
  import type { ShellCommandNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: ShellCommandNodeData;
  }

  let { id, data }: Props = $props();
  const { updateNodeData } = useSvelteFlow();

  function patch(next: Partial<ShellCommandNodeData>) {
    updateNodeData(id, next);
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Shell Command",
    subline: data.subline ?? "Run a system command",
  }}
>
  <div class="input-container">
    <span>Command:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="command"
      value={data.command ?? ""}
      placeholder="program --flag"
      onInput={(command) => patch({ command })}
    />
  </div>

  <div class="input-container">
    <span>Timeout (ms):</span>
    <ValueBackedInput
      nodeId={id}
      inputName="timeoutMs"
      inputmode="numeric"
      value={data.timeoutMs ?? 0}
      placeholder="0"
      onInput={(value) => patch({ timeoutMs: Number(value) || 0 })}
    />
  </div>

  <div class="input-container">
    <span>Env:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="env"
      value={data.env ?? ""}
      placeholder="FOO=bar"
      onInput={(env) => patch({ env })}
    />
  </div>

  <div class="input-container">
    <span>Cwd:</span>
    <ValueBackedInput
      nodeId={id}
      inputName="cwd"
      value={data.cwd ?? ""}
      placeholder="/path"
      onInput={(cwd) => patch({ cwd })}
    />
  </div>

  <label class="checkbox-row nodrag">
    <input
      type="checkbox"
      checked={data.runInBackground ?? false}
      onchange={(event) => patch({ runInBackground: event.currentTarget.checked })}
    />
    <span>Run in background</span>
  </label>

  <div class="output-labels">
    <span>Exit</span>
    <span>Stdout</span>
    <span>Stderr</span>
  </div>
</BaseNode>

{#each ["command", "timeoutMs", "env", "cwd"] as inputName, index}
  <Handle
    type="target"
    position={Position.Top}
    id={valueHandle(inputName)}
    style="left: {20 + index * 20}%; border-color: #38d0ff; background: #102a36"
  />
{/each}

<Handle
  type="source"
  position={Position.Bottom}
  id={valueHandle("exitCode")}
  style="left: 25%; border-color: #38d0ff; background: #102a36"
/>
<Handle
  type="source"
  position={Position.Bottom}
  id={valueHandle("stdout")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
<Handle
  type="source"
  position={Position.Bottom}
  id={valueHandle("stderr")}
  style="left: 75%; border-color: #38d0ff; background: #102a36"
/>

<style>
  .checkbox-row {
    align-items: center;
    color: #e0e0e0;
    display: flex;
    font-size: 0.78rem;
    gap: 0.4rem;
    margin: 0.45rem 0.5rem 0;
  }

  .checkbox-row input {
    accent-color: #a853ba;
    margin: 0;
  }

  .output-labels {
    color: #8fd4ff;
    display: grid;
    font-size: 0.7rem;
    gap: 0.35rem;
    grid-template-columns: repeat(3, 1fr);
    margin: 0.55rem 0.5rem 0;
    text-align: center;
  }
</style>
