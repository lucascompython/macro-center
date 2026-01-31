<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import NumberInput from '$lib/components/NumberInput.svelte';
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

	function handleDelayChange(value: number) {
		updateNodeData(id, { delay: value });
	}

	function handleCancelKeyChange(e: Event & { currentTarget: HTMLSpanElement }) {
		updateNodeData(id, { cancelKey: e.currentTarget.innerText });
	}

	let delay = $state(data.delay ?? 0.5);
	$effect(() => {
		handleDelayChange(delay);
	});
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

	<div class="input-container">
		<span>Delay (s):</span>
		<NumberInput bind:value={delay} maxWidth="80px" />
	</div>

	<div class="input-container">
		<span>Cancel key:</span>
		<span
			class="nodrag optional-placeholder"
			contenteditable="true"
			role="textbox"
			oninput={handleCancelKeyChange}>{data.cancelKey ?? ''}</span
		>
	</div>
</BaseNode>
