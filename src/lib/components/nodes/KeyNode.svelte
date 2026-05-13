<script lang="ts">
  import { Handle, Position, useSvelteFlow, useStore } from "@xyflow/svelte";
  import { getContext } from "svelte";
  import BaseNode from "./BaseNode.svelte";
  import { VALUE_SOURCE_CONTEXT, type ValueSourceContext } from "$lib/editor-context";
  import { valueHandle } from "$lib/graph";
  import { ActionMode, type KeyNodeData } from "$lib/types";
  import { resolveConnectedValuePreview } from "$lib/value-sources";

  interface Props {
    id: string;
    data: KeyNodeData;
  }

  let { id, data }: Props = $props();

  const { updateNodeData } = useSvelteFlow();
  const store = useStore();
  const valueSources = getContext<ValueSourceContext | undefined>(VALUE_SOURCE_CONTEXT);

  const connectedKey = $derived.by(() =>
    resolveConnectedValuePreview(store, id, "key", {
      variables: valueSources?.getVariables() ?? [],
      snapshot: valueSources?.getSnapshot() ?? {},
    }),
  );

  let recording = $state(false);

  function startRecording() {
    recording = true;
  }

  function stopRecording(key: string) {
    recording = false;
    updateNodeData(id, { key });
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!recording) return;

    e.preventDefault();
    e.stopPropagation();

    let finalKey = e.key;
    if (finalKey === " ") finalKey = "Space";
    if (finalKey === "Meta") finalKey = "Super";

    stopRecording(finalKey);
  }

  function handleBlur() {
    if (recording) {
      recording = false;
    }
  }

  function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
    updateNodeData(id, { mode: e.currentTarget.value as ActionMode });
  }
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Key Press",
    subline: data.subline ?? "Press/release/click a key",
  }}
>
  <div class="input-container key-container">
    <span>Key:</span>

    {#if connectedKey}
      <input
        class="nodrag key-source"
        type="text"
        value={connectedKey.text}
        readonly
        title={connectedKey.title}
      />
    {:else}
      <button
        class="nodrag keybind-btn"
        class:recording
        onkeydown={handleKeyDown}
        onclick={startRecording}
        onblur={handleBlur}
      >
        {#if recording}
          <div class="recording-dot"></div>
        {/if}
        <span class="keybind-label">{recording ? "..." : data.key || "Click to record"}</span>
      </button>
    {/if}
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
  id={valueHandle("key")}
  style="left: 50%; border-color: #38d0ff; background: #102a36"
/>

<style>
  .key-container {
    align-items: center;
    display: flex;
  }

  .keybind-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.2rem 0.5rem;
    border-radius: 0.25rem;
    border: 1px solid #555;
    background: #3a3b3d;
    color: #e0e0e0;
    font-family: inherit;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s ease;
    text-align: left;
    margin-left: 0.5rem;
    min-width: 5rem;
    max-width: 140px;
  }

  input.key-source {
    background: rgba(233, 42, 103, 0.15);
    border: 1px solid rgba(255, 138, 0, 0.48);
    border-radius: 0.25rem;
    color: #ffd7a6;
    cursor: default;
    font-family: inherit;
    font-size: 0.85rem;
    margin-left: 0.5rem;
    max-width: 140px;
    min-width: 5rem;
    outline: none;
    padding: 0.2rem 0.5rem;
  }

  .keybind-btn:hover {
    border-color: #888;
    background: #444546;
  }

  .keybind-btn:focus {
    outline: none;
    border-color: #e92a67;
    box-shadow: 0 0 0 1px rgba(233, 42, 103, 0.3);
  }

  .keybind-btn.recording {
    border-color: #e92a67;
    background: #3a2029;
    animation: pulse-border 1.5s ease-in-out infinite;
  }

  @keyframes pulse-border {
    0%,
    100% {
      border-color: #e92a67;
    }
    50% {
      border-color: #ff4d8a;
    }
  }

  .recording-dot {
    display: block;
    flex: 0 0 6px !important;
    width: 6px !important;
    height: 6px !important;
    min-width: 6px !important;
    max-width: 6px !important;
    min-height: 6px !important;
    max-height: 6px !important;
    padding: 0 !important;
    margin: 0 !important;
    border-radius: 50% !important;
    background: #e92a67 !important;
    animation: blink 1s ease-in-out infinite;
  }

  @keyframes blink {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.3;
    }
  }

  .keybind-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
