<script lang="ts">
  import { useStore } from "@xyflow/svelte";
  import { getContext } from "svelte";
  import { VALUE_SOURCE_CONTEXT, type ValueSourceContext } from "$lib/editor-context";
  import { resolveConnectedValuePreview } from "$lib/value-sources";

  interface ValueBackedSelectOption {
    value: string;
    label: string;
  }

  interface Props {
    nodeId: string;
    inputName: string;
    value: string;
    options: ValueBackedSelectOption[];
    onChange: (value: string) => void;
  }

  let { nodeId, inputName, value, options, onChange }: Props = $props();

  const store = useStore();
  const valueSources = getContext<ValueSourceContext | undefined>(VALUE_SOURCE_CONTEXT);

  const connected = $derived.by(() =>
    resolveConnectedValuePreview(store, nodeId, inputName, {
      variables: valueSources?.getVariables() ?? [],
      snapshot: valueSources?.getSnapshot() ?? {},
    }),
  );

  function handleChange(event: Event & { currentTarget: HTMLSelectElement }) {
    onChange(event.currentTarget.value);
  }
</script>

{#if connected}
  <input
    class="nodrag value-backed-select-proxy connected"
    type="text"
    value={connected.text}
    readonly
    title={connected.title}
  />
{:else}
  <select class="nodrag value-backed-select" {value} onchange={handleChange}>
    {#each options as option}
      <option value={option.value}>{option.label}</option>
    {/each}
  </select>
{/if}

<style>
  .value-backed-select,
  .value-backed-select-proxy {
    background: #4c4d4f;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: white;
    font-family: inherit;
    font-size: 0.9rem;
    outline: none;
    padding: 0.2rem 0.4rem;
  }

  .value-backed-select {
    appearance: none;
    cursor: pointer;
    padding-right: 2rem;
  }

  .value-backed-select:focus {
    border-color: #e92a67;
  }

  .value-backed-select-proxy.connected {
    border-color: rgba(255, 138, 0, 0.48);
    background: rgba(233, 42, 103, 0.15);
    color: #ffd7a6;
    cursor: default;
    max-width: 180px;
    min-width: 4rem;
  }
</style>
