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

	function handleTextChange(e: Event & { currentTarget: HTMLInputElement }) {
		updateNodeData(id, { text: e.currentTarget.value });
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
		<input class="nodrag" type="text" value={data.text ?? 'Hello World!'} oninput={handleTextChange} />
	</div>
</BaseNode>
