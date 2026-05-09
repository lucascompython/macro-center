<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import ValueBackedInput from '$lib/components/ValueBackedInput.svelte';
	import { valueHandle } from '$lib/graph';
	import type { RepeatLoopNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: RepeatLoopNodeData;
	}

	let { id, data }: Props = $props();
	const { updateNodeData } = useSvelteFlow();

	function handleIterationsInput(value: string) {
		updateNodeData(id, { iterations: Number(value) || 0 });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Repeat',
		subline: data.subline ?? 'Run body N times',
		enableSource: false
	}}
>
	<div class="input-container">
		<span>Times:</span>
		<ValueBackedInput
			nodeId={id}
			inputName="iterations"
			inputmode="numeric"
			value={data.iterations ?? 3}
			onInput={handleIterationsInput}
		/>
	</div>
	<div class="input-container">
		<span>Index:</span>
		<input
			class="nodrag"
			type="text"
			value={data.indexVariable ?? 'index'}
			oninput={(event) => updateNodeData(id, { indexVariable: event.currentTarget.value })}
		/>
	</div>
	<div class="loop-outputs">
		<span>Body</span>
		<span>Done</span>
	</div>
</BaseNode>

<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('iterations')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
<Handle type="source" position={Position.Right} id="body" style="top: 58%; border-color: #ffc857" />
<Handle type="source" position={Position.Right} id="done" style="top: 82%; border-color: #27d209" />

<style>
	.loop-outputs {
		color: #d8d8d8;
		display: flex;
		flex-direction: column;
		font-size: 0.75rem;
		gap: 0.5rem;
		margin: 0.45rem 0.5rem 0;
		text-align: right;
	}
</style>
