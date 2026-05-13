<script lang="ts">
  import { coerceValueByType, defaultValueForType, parseLiteral, valueToInput } from "$lib/logic";
  import type { MacroValue, ValueType, VariableDefinition, VariableScope } from "$lib/types";

  interface Props {
    variables: VariableDefinition[];
    snapshot?: Record<string, MacroValue>;
  }

  let { variables = $bindable([]), snapshot = {} }: Props = $props();

  const valueTypes: ValueType[] = [
    "boolean",
    "number",
    "text",
    "key",
    "mouseButton",
    "point",
    "list",
  ];
  const scopes: VariableScope[] = ["macro", "persistent"];

  function addVariable() {
    const nextIndex = variables.length + 1;
    variables = [
      ...variables,
      {
        id: crypto.randomUUID(),
        name: `var_${nextIndex}`,
        type: "number",
        defaultValue: 0,
        scope: "macro",
      },
    ];
  }

  function updateVariable(id: string, patch: Partial<VariableDefinition>) {
    variables = variables.map((variable) =>
      variable.id === id ? { ...variable, ...patch } : variable,
    );
  }

  function deleteVariable(id: string) {
    variables = variables.filter((variable) => variable.id !== id);
  }

  function changeType(variable: VariableDefinition, type: ValueType) {
    updateVariable(variable.id, {
      type,
      defaultValue: coerceValueByType(variable.defaultValue ?? defaultValueForType(type), type),
    });
  }

  function changeDefault(variable: VariableDefinition, raw: string) {
    updateVariable(variable.id, {
      defaultValue: coerceValueByType(parseLiteral(raw), variable.type),
    });
  }
</script>

<section class="variables-panel">
  <div class="panel-header">
    <div>
      <h2>Variables</h2>
      <p>Reusable values for this macro</p>
    </div>
    <button class="icon-btn" onclick={addVariable} title="Add variable">+</button>
  </div>

  {#if variables.length === 0}
    <div class="empty">No variables yet.</div>
  {:else}
    <div class="variable-list">
      {#each variables as variable (variable.id)}
        <div class="variable-row">
          <input
            class="name"
            type="text"
            value={variable.name}
            oninput={(event) =>
              updateVariable(variable.id, {
                name: event.currentTarget.value,
              })}
          />

          <select
            value={variable.type}
            onchange={(event) => changeType(variable, event.currentTarget.value as ValueType)}
          >
            {#each valueTypes as valueType}
              <option value={valueType}>{valueType}</option>
            {/each}
          </select>

          <select
            value={variable.scope}
            onchange={(event) =>
              updateVariable(variable.id, {
                scope: event.currentTarget.value as VariableScope,
              })}
          >
            {#each scopes as scope}
              <option value={scope}>{scope}</option>
            {/each}
          </select>

          <input
            class="default-value"
            type="text"
            value={valueToInput(variable.defaultValue)}
            onchange={(event) => changeDefault(variable, event.currentTarget.value)}
          />

          <div class="live-value" title="Current run value">
            {valueToInput(snapshot[variable.name] ?? variable.defaultValue)}
          </div>

          <button
            class="delete-btn"
            onclick={() => deleteVariable(variable.id)}
            title="Delete variable"
          >
            Delete
          </button>
        </div>
      {/each}
    </div>
  {/if}
</section>

<style>
  .variables-panel {
    background: rgba(18, 18, 18, 0.96);
    border: 1px solid #3e3e3e;
    border-radius: 6px;
    color: #e0e0e0;
    font-family: "Inter", sans-serif;
    min-width: 0;
    width: max-content;
    padding: 0.75rem;
  }

  .panel-header {
    align-items: center;
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    margin-bottom: 0.6rem;
  }

  h2 {
    font-size: 0.95rem;
    margin: 0;
  }

  p {
    color: #888;
    font-size: 0.75rem;
    margin: 0.15rem 0 0;
  }

  .icon-btn,
  .delete-btn {
    background: #2c2d2f;
    border: 1px solid #4a4a4a;
    border-radius: 4px;
    color: #f1f1f1;
    cursor: pointer;
    font: inherit;
    padding: 0.25rem 0.5rem;
  }

  .variable-list {
    display: grid;
    gap: 0.4rem;
    max-height: 280px;
    overflow: auto;
  }

  .variable-row {
    display: grid;
    grid-template-columns: 72px 84px 84px 72px 72px 72px;
    gap: 0.35rem;
  }

  input,
  select,
  .live-value {
    background: #2c2d2f;
    border: 1px solid #414141;
    border-radius: 4px;
    color: #f1f1f1;
    font: inherit;
    font-size: 0.75rem;
    min-width: 0;
    padding: 0.28rem 0.35rem;
  }

  input.name,
  input.default-value,
  .live-value {
    width: 72px;
  }

  .live-value {
    color: #8fd4ff;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    align-content: center;
  }

  .empty {
    color: #888;
    font-size: 0.8rem;
    padding: 0.5rem 0.1rem 0.1rem;
  }
</style>
