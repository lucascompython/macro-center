<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import NumberInput from '$lib/components/NumberInput.svelte';
	import type { DelayNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: DelayNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	let delay = $state(data.delay ?? 1000);

	$effect(() => {
		updateNodeData(id, { delay });
	});
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
		<NumberInput bind:value={delay} intOnly={true} maxWidth="100px" />
	</div>
</BaseNode>
