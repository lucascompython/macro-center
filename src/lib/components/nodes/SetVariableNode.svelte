<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import ValueBackedInput from '$lib/components/ValueBackedInput.svelte';
	import {
		coerceValueByType,
		defaultValueForType,
		parseLiteral,
		valueToInput
	} from '$lib/logic';
	import { valueHandle } from '$lib/graph';
	import type { SetVariableNodeData, ValueType, VariableScope } from '$lib/types';

	interface Props {
		id: string;
		data: SetVariableNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();
	const valueTypes: ValueType[] = ['boolean', 'number', 'text', 'key', 'mouseButton', 'point', 'list'];
	const scopes: VariableScope[] = ['macro', 'persistent'];

	function patch(next: Partial<SetVariableNodeData>) {
		updateNodeData(id, next);
	}

	function handleTypeChange(event: Event & { currentTarget: HTMLSelectElement }) {
		const valueType = event.currentTarget.value as ValueType;
		patch({
			valueType,
			value: coerceValueByType(data.value ?? defaultValueForType(valueType), valueType)
		});
	}

	function handleValueChange(rawValue: string) {
		patch({
			value: coerceValueByType(parseLiteral(rawValue), data.valueType ?? 'text')
		});
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Set Variable',
		subline: data.subline ?? 'Assign a value'
	}}
>
	<div class="input-container">
		<span>Name:</span>
		<input
			class="nodrag"
			type="text"
			value={data.variableName ?? 'value'}
			oninput={(event) => patch({ variableName: event.currentTarget.value })}
		/>
	</div>

	<div class="input-container">
		<span>Type:</span>
		<select class="nodrag" value={data.valueType ?? 'text'} onchange={handleTypeChange}>
			{#each valueTypes as valueType}
				<option value={valueType}>{valueType}</option>
			{/each}
		</select>
	</div>

	<div class="input-container">
		<span>Scope:</span>
		<select
			class="nodrag"
			value={data.scope ?? 'macro'}
			onchange={(event) => patch({ scope: event.currentTarget.value as VariableScope })}
		>
			{#each scopes as scope}
				<option value={scope}>{scope}</option>
			{/each}
		</select>
	</div>

	<div class="input-container">
		<span>Value:</span>
		<ValueBackedInput
			nodeId={id}
			inputName="value"
			value={valueToInput(data.value ?? defaultValueForType(data.valueType ?? 'text'))}
			onChange={handleValueChange}
		/>
	</div>
</BaseNode>

<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('value')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
