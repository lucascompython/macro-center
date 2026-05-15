<script lang="ts">
  import {
    conditionOperatorLabels,
    defaultConditionExpression,
    expressionNeedsRightOperand,
    operandToInput,
    parseOperandInput,
  } from "$lib/logic";
  import ValueBackedInput from "$lib/components/ValueBackedInput.svelte";
  import type { ConditionExpression, ConditionOperator } from "$lib/types";

  interface Props {
    expression?: ConditionExpression;
    onChange: (expression: ConditionExpression) => void;
    compact?: boolean;
    nodeId?: string;
    leftInputName?: string;
    rightInputName?: string;
  }

  let {
    expression = defaultConditionExpression,
    onChange,
    compact = false,
    nodeId = undefined,
    leftInputName = "left",
    rightInputName = "right",
  }: Props = $props();

  const operators = Object.entries(conditionOperatorLabels) as [ConditionOperator, string][];
  const needsRight = $derived(expressionNeedsRightOperand(expression.operator));

  function update(next: Partial<ConditionExpression>) {
    onChange({
      ...expression,
      ...next,
    });
  }

  function handleLeftInput(value: string) {
    update({ left: parseOperandInput(value) });
  }

  function handleOperatorChange(event: Event & { currentTarget: HTMLSelectElement }) {
    update({ operator: event.currentTarget.value as ConditionOperator });
  }

  function handleRightInput(value: string) {
    update({ right: parseOperandInput(value) });
  }
</script>

<div class:compact class="condition-editor nodrag">
  {#if nodeId}
    <ValueBackedInput
      {nodeId}
      inputName={leftInputName}
      value={operandToInput(expression.left)}
      placeholder="value"
      onInput={handleLeftInput}
    />
  {:else}
    <input
      type="text"
      value={operandToInput(expression.left)}
      placeholder="value"
      oninput={(event) => handleLeftInput(event.currentTarget.value)}
    />
  {/if}

  <select value={expression.operator} onchange={handleOperatorChange}>
    {#each operators as [operator, label]}
      <option value={operator}>{label}</option>
    {/each}
  </select>

  {#if needsRight}
    {#if nodeId}
      <ValueBackedInput
        {nodeId}
        inputName={rightInputName}
        value={operandToInput(expression.right)}
        placeholder="value"
        onInput={handleRightInput}
      />
    {:else}
      <input
        type="text"
        value={operandToInput(expression.right)}
        placeholder="value"
        oninput={(event) => handleRightInput(event.currentTarget.value)}
      />
    {/if}
  {/if}
</div>

<style>
  .condition-editor {
    display: grid;
    grid-template-columns: minmax(5rem, 1fr);
    gap: 0.35rem;
    padding: 0.35rem 0.5rem;
  }

  .condition-editor.compact {
    padding: 0;
  }

  input,
  select {
    background: #4c4d4f;
    border: 1px solid transparent;
    border-radius: 0.25rem;
    color: white;
    font-family: inherit;
    font-size: 0.82rem;
    min-width: 0;
    outline: none;
    padding: 0.18rem 0.3rem;
  }

  select {
    appearance: none;
    cursor: pointer;
  }

  input:focus,
  select:focus {
    border-color: #e92a67;
  }

  .condition-editor :global(.value-backed-input) {
    font-size: 0.82rem;
    max-width: none;
    min-width: 0;
    padding: 0.18rem 0.3rem;
    width: 100%;
  }
</style>
