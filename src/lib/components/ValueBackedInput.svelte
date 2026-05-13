<script lang="ts">
  import { useStore } from "@xyflow/svelte";
  import { getContext } from "svelte";
  import { VALUE_SOURCE_CONTEXT, type ValueSourceContext } from "$lib/editor-context";
  import { valueToInput } from "$lib/logic";
  import type { MacroValue } from "$lib/types";
  import { resolveConnectedValuePreview } from "$lib/value-sources";

  type InputMode = "none" | "text" | "search" | "tel" | "url" | "email" | "numeric" | "decimal";

  interface Props {
    nodeId: string;
    inputName: string;
    value: MacroValue;
    inputmode?: InputMode;
    placeholder?: string;
    onInput?: (raw: string) => void;
    onChange?: (raw: string) => void;
  }

  let {
    nodeId,
    inputName,
    value,
    inputmode = undefined,
    placeholder = undefined,
    onInput = undefined,
    onChange = undefined,
  }: Props = $props();

  const store = useStore();
  const valueSources = getContext<ValueSourceContext | undefined>(VALUE_SOURCE_CONTEXT);

  const connected = $derived.by(() =>
    resolveConnectedValuePreview(store, nodeId, inputName, {
      variables: valueSources?.getVariables() ?? [],
      snapshot: valueSources?.getSnapshot() ?? {},
    }),
  );
  const connectedValue = $derived(Boolean(connected));
  const displayValue = $derived(connected ? connected.text : valueToInput(value));

  function handleInput(event: Event & { currentTarget: HTMLInputElement }) {
    if (connectedValue) return;
    onInput?.(event.currentTarget.value);
  }

  function handleChange(event: Event & { currentTarget: HTMLInputElement }) {
    if (connectedValue) return;
    onChange?.(event.currentTarget.value);
  }
</script>

<input
  class="nodrag value-backed-input"
  class:connected={connectedValue}
  type="text"
  {inputmode}
  {placeholder}
  value={displayValue}
  readonly={connectedValue}
  title={connected?.title}
  oninput={handleInput}
  onchange={handleChange}
/>

<style>
  .value-backed-input {
    background: #4c4d4f;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: white;
    cursor: text;
    font-family: inherit;
    font-size: 0.9rem;
    font-weight: 400;
    max-width: 180px;
    min-width: 4rem;
    outline: none;
    padding: 0.1rem 0.25rem;
    transition:
      background 0.2s ease,
      border 0.2s ease,
      color 0.2s ease;
  }

  .value-backed-input:focus {
    border-color: #e92a67;
  }

  .value-backed-input.connected {
    background: rgba(233, 42, 103, 0.15);
    border-color: rgba(255, 138, 0, 0.48);
    color: #ffd7a6;
    cursor: default;
  }
</style>
