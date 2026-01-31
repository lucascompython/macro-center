<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
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
</BaseNode>
