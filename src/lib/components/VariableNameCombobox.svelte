<script lang="ts">
	import { getContext } from 'svelte';
	import { VALUE_SOURCE_CONTEXT, type ValueSourceContext } from '$lib/editor-context';

	interface Props {
		value: string;
		onChange: (value: string) => void;
		placeholder?: string;
	}

	let { value, onChange, placeholder = 'value' }: Props = $props();

	const valueSources = getContext<ValueSourceContext | undefined>(VALUE_SOURCE_CONTEXT);
	const variables = $derived(valueSources?.getVariables() ?? []);
	const optionId = $derived(`variables-${value.replace(/[^a-zA-Z0-9_-]/g, '-') || 'name'}`);

	function handleInput(event: Event & { currentTarget: HTMLInputElement }) {
		onChange(event.currentTarget.value);
	}
</script>

<div class="variable-combobox">
	<input
		class="nodrag"
		type="text"
		list={optionId}
		{placeholder}
		{value}
		oninput={handleInput}
	/>
	<datalist id={optionId}>
		{#each variables as variable (variable.id)}
			<option value={variable.name}>{variable.type}</option>
		{/each}
	</datalist>
</div>

<style>
	.variable-combobox {
		position: relative;
	}

	input {
		background: #4c4d4f;
		border: 1px solid transparent;
		border-radius: 0.25rem;
		color: white;
		cursor: text;
		font-family: inherit;
		font-size: 0.9rem;
		font-weight: 400;
		margin-left: 0.5rem;
		max-width: 180px;
		min-width: 4rem;
		outline: none;
		padding: 0.1rem 0.25rem;
		transition: border 0.2s ease;
	}

	input:focus {
		border-color: #e92a67;
	}
</style>
