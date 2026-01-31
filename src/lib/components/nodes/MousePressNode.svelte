<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import { ActionMode, MouseButton, type MousePressNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: MousePressNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleButtonChange(e: Event & { currentTarget: HTMLSelectElement }) {
		updateNodeData(id, { button: e.currentTarget.value as MouseButton });
	}

	function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
		updateNodeData(id, { mode: e.currentTarget.value as ActionMode });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Mouse Press',
		subline: data.subline ?? 'Simulate mouse button action'
	}}
>
	<div class="input-container">
		<span>Button:</span>
		<select class="nodrag" value={data.button ?? MouseButton.LEFT} onchange={handleButtonChange}>
			<option value={MouseButton.LEFT}>Left</option>
			<option value={MouseButton.RIGHT}>Right</option>
			<option value={MouseButton.MIDDLE}>Middle</option>
			<option value={MouseButton.MOUSE4}>Mouse4</option>
			<option value={MouseButton.MOUSE5}>Mouse5</option>
		</select>
	</div>

	<div class="input-container">
		<span>Mode:</span>
		<select class="nodrag" value={data.mode ?? ActionMode.CLICK} onchange={handleModeChange}>
			<option value={ActionMode.CLICK}>Click</option>
			<option value={ActionMode.PRESS}>Press</option>
			<option value={ActionMode.RELEASE}>Release</option>
		</select>
	</div>
</BaseNode>
