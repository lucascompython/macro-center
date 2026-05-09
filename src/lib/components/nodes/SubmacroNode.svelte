<script lang="ts">
	import { Handle, Position } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import { submacroOutputHandle, valueHandle } from '$lib/graph';
	import type { SubmacroNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: SubmacroNodeData;
	}

	let { id, data }: Props = $props();

	const triggerOutputs = $derived(data.triggerOutputs?.length ? data.triggerOutputs : [{ id: 'done', label: 'Done' }]);
	const valueInputs = $derived(data.valueInputs ?? []);
	const valueOutputs = $derived(data.valueOutputs ?? []);

	function topFor(index: number, total: number, start = 48, span = 38) {
		if (total <= 1) return `${start + span / 2}%`;
		return `${start + (span * index) / (total - 1)}%`;
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Submacro',
		subline: data.subline ?? 'Call visible subflow',
		enableSource: false
	}}
>
	{#if valueInputs.length > 0}
		<div class="port-group">
			<span class="port-heading">Inputs</span>
			{#each valueInputs as input}
				<span>{input.label}</span>
			{/each}
		</div>
	{/if}

	<div class="port-grid">
		<div>
			<span class="port-heading">Run</span>
		</div>
		<div>
			<span class="port-heading">Outputs</span>
			{#each triggerOutputs as output}
				<span>{output.label}</span>
			{/each}
			{#each valueOutputs as output}
				<span class="value-output">{output.label}</span>
			{/each}
		</div>
	</div>
</BaseNode>

{#each valueInputs as input, index}
	<Handle
		type="target"
		position={Position.Top}
		id={valueHandle(input.id)}
		style="left: {topFor(index, valueInputs.length, 22, 56)}; border-color: #38d0ff; background: #102a36"
	/>
{/each}

{#each triggerOutputs as output, index}
	<Handle
		type="source"
		position={Position.Right}
		id={submacroOutputHandle(output.id)}
		style="top: {topFor(index, triggerOutputs.length, 46, 24)}; border-color: #27d209"
	/>
{/each}

{#each valueOutputs as output, index}
	<Handle
		type="source"
		position={Position.Bottom}
		id={valueHandle(output.id)}
		style="left: {topFor(index, valueOutputs.length, 28, 44)}; border-color: #38d0ff; background: #102a36"
	/>
{/each}

<style>
	.port-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 0.75rem;
		padding: 0.35rem 0.5rem;
	}

	.port-group,
	.port-grid > div {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.port-group {
		padding: 0.35rem 0.5rem 0;
	}

	.port-heading {
		color: #888;
		font-size: 0.7rem;
		text-transform: uppercase;
	}

	span {
		font-size: 0.78rem;
	}

	.value-output {
		color: #ffb000;
	}
</style>
