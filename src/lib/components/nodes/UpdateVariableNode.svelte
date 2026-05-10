<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import ValueBackedInput from '$lib/components/ValueBackedInput.svelte';
	import VariableNameCombobox from '$lib/components/VariableNameCombobox.svelte';
	import { parseLiteral, valueToInput } from '$lib/logic';
	import { valueHandle } from '$lib/graph';
	import type { UpdateVariableNodeData, UpdateVariableOperation } from '$lib/types';

	interface Props {
		id: string;
		data: UpdateVariableNodeData;
	}

	let { id, data }: Props = $props();
	const { updateNodeData } = useSvelteFlow();
	const operations: UpdateVariableOperation[] = [
		'increment',
		'decrement',
		'set',
		'append',
		'toggle',
		'clear'
	];

	function patch(next: Partial<UpdateVariableNodeData>) {
		updateNodeData(id, next);
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Update Variable',
		subline: data.subline ?? 'Change an existing value'
	}}
>
	<div class="input-container">
		<span>Name:</span>
		<VariableNameCombobox
			value={data.variableName ?? 'value'}
			onChange={(variableName) => patch({ variableName })}
		/>
	</div>

	<div class="input-container">
		<span>Op:</span>
		<select
			class="nodrag"
			value={data.operation ?? 'increment'}
			onchange={(event) => patch({ operation: event.currentTarget.value as UpdateVariableOperation })}
		>
			{#each operations as operation}
				<option value={operation}>{operation}</option>
			{/each}
		</select>
	</div>

	<div class="input-container">
		<span>Value:</span>
		<ValueBackedInput
			nodeId={id}
			inputName="value"
			value={valueToInput(data.value ?? 1)}
			onChange={(value) => patch({ value: parseLiteral(value) })}
		/>
	</div>
</BaseNode>

<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('value')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
