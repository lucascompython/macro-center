<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import NumberInput from '$lib/components/NumberInput.svelte';
	import { CoordinateMode, type MouseMoveNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: MouseMoveNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
		updateNodeData(id, { coordinateMode: e.currentTarget.value as CoordinateMode });
	}

	let x = $state(data.x ?? 0);
	let y = $state(data.y ?? 0);

	$effect(() => {
		updateNodeData(id, { x });
	});

	$effect(() => {
		updateNodeData(id, { y });
	});
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Move Mouse',
		subline: data.subline ?? 'Move cursor to position'
	}}
>
	<div class="input-container">
		<span>X:</span>
		<NumberInput bind:value={x} intOnly={true} maxWidth="80px" />
	</div>

	<div class="input-container">
		<span>Y:</span>
		<NumberInput bind:value={y} intOnly={true} maxWidth="80px" />
	</div>

	<div class="input-container">
		<span>Mode:</span>
		<select
			class="nodrag"
			value={data.coordinateMode ?? CoordinateMode.ABSOLUTE}
			onchange={handleModeChange}
		>
			<option value={CoordinateMode.ABSOLUTE}>Absolute</option>
			<option value={CoordinateMode.RELATIVE}>Relative</option>
		</select>
	</div>
</BaseNode>
