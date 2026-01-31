<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import NumberInput from '$lib/components/NumberInput.svelte';
	import { Axis, type ScrollMouseNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: ScrollMouseNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleAxisChange(e: Event & { currentTarget: HTMLSelectElement }) {
		updateNodeData(id, { axis: e.currentTarget.value as Axis });
	}

	let amount = $state(data.amount ?? 3);

	$effect(() => {
		updateNodeData(id, { amount });
	});
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Scroll Mouse',
		subline: data.subline ?? 'Scroll the mouse wheel'
	}}
>
	<div class="input-container">
		<span>Axis:</span>
		<select class="nodrag" value={data.axis ?? Axis.VERTICAL} onchange={handleAxisChange}>
			<option value={Axis.VERTICAL}>Vertical</option>
			<option value={Axis.HORIZONTAL}>Horizontal</option>
		</select>
	</div>

	<div class="input-container">
		<span>Amount:</span>
		<NumberInput bind:value={amount} intOnly={true} maxWidth="80px" />
	</div>
</BaseNode>
