<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import { valueHandle } from '$lib/graph';
	import type { GetVariableNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: GetVariableNodeData;
	}

	let { id, data }: Props = $props();
	const { updateNodeData } = useSvelteFlow();
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Get Variable',
		subline: data.subline ?? 'Read a value',
		enableTarget: false,
		enableSource: false
	}}
>
	<div class="input-container">
		<span>Name:</span>
		<input
			class="nodrag"
			type="text"
			value={data.variableName ?? 'value'}
			oninput={(event) => updateNodeData(id, { variableName: event.currentTarget.value })}
		/>
	</div>
</BaseNode>

<Handle
	type="source"
	position={Position.Bottom}
	id={valueHandle('value')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
