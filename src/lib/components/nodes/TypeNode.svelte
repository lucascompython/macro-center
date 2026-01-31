<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import NumberInput from '$lib/components/NumberInput.svelte';
	import type { TypeNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: TypeNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleTextChange(e: Event & { currentTarget: HTMLSpanElement }) {
		updateNodeData(id, { text: e.currentTarget.innerText });
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
		title: data.title ?? 'Type Text',
		subline: data.subline ?? 'Types a given text'
	}}
>
	<div class="input-container">
		<span>Text:</span>
		<span class="nodrag" contenteditable="true" role="textbox" oninput={handleTextChange}
			>{data.text ?? 'Hello World!'}</span
		>
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
