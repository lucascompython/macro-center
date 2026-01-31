<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import { ActionMode, type KeyNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: KeyNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleKeyChange(e: Event & { currentTarget: HTMLSpanElement }) {
		updateNodeData(id, { key: e.currentTarget.innerText });
	}

	function handleModeChange(e: Event & { currentTarget: HTMLSelectElement }) {
		updateNodeData(id, { mode: e.currentTarget.value as ActionMode });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Key Press',
		subline: data.subline ?? 'Press/release/click a key'
	}}
>
	<div class="input-container">
		<span>Key:</span>
		<span class="nodrag" contenteditable="true" role="textbox" oninput={handleKeyChange}
			>{data.key ?? 'Enter'}</span
		>
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
