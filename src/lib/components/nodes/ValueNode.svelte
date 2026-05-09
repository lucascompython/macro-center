<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import {
		coerceValueByType,
		defaultValueForType,
		parseLiteral,
		valueToInput
	} from '$lib/logic';
	import { valueHandle } from '$lib/graph';
	import type { ValueNodeData, ValueType } from '$lib/types';

	interface Props {
		id: string;
		data: ValueNodeData;
	}

	let { id, data }: Props = $props();
	const { updateNodeData } = useSvelteFlow();
	const valueTypes: ValueType[] = ['boolean', 'number', 'text', 'key', 'mouseButton', 'point', 'list'];

	function patch(next: Partial<ValueNodeData>) {
		updateNodeData(id, next);
	}

	function handleTypeChange(event: Event & { currentTarget: HTMLSelectElement }) {
		const valueType = event.currentTarget.value as ValueType;
		patch({
			valueType,
			value: coerceValueByType(data.value ?? defaultValueForType(valueType), valueType)
		});
	}

	function handleValueChange(event: Event & { currentTarget: HTMLInputElement }) {
		patch({
			value: coerceValueByType(parseLiteral(event.currentTarget.value), data.valueType ?? 'text')
		});
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Value',
		subline: data.subline ?? 'Literal output',
		enableTarget: false,
		enableSource: false
	}}
>
	<div class="input-container">
		<span>Type:</span>
		<select class="nodrag" value={data.valueType ?? 'text'} onchange={handleTypeChange}>
			{#each valueTypes as valueType}
				<option value={valueType}>{valueType}</option>
			{/each}
		</select>
	</div>

	<div class="input-container">
		<span>Value:</span>
		<input
			class="nodrag"
			type="text"
			value={valueToInput(data.value ?? defaultValueForType(data.valueType ?? 'text'))}
			onchange={handleValueChange}
		/>
	</div>
</BaseNode>

<Handle
	type="source"
	position={Position.Bottom}
	id={valueHandle('value')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
