<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import ValueBackedInput from '$lib/components/ValueBackedInput.svelte';
	import { valueHandle } from '$lib/graph';
	import type { DelayNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: DelayNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleDelayInput(value: string) {
		updateNodeData(id, { delay: Number(value) || 0 });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Delay',
		subline: data.subline ?? 'Wait before continuing'
	}}
>
	<div class="input-container">
		<span>Delay (ms):</span>
		<ValueBackedInput
			nodeId={id}
			inputName="delay"
			inputmode="numeric"
			value={data.delay ?? 1000}
			onInput={handleDelayInput}
		/>
	</div>
</BaseNode>

<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('delay')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
