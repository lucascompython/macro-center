<script lang="ts">
	import { useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import type { KeyBindNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: KeyBindNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	function handleKeyBindChange(e: Event & { currentTarget: HTMLSpanElement }) {
		updateNodeData(id, { keyBind: e.currentTarget.innerText });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Key Bind',
		subline: data.subline ?? 'Trigger macro with key combination',
		enableTarget: false,
		enableSource: true
	}}
>
	<div class="input-container">
		<span>Key Bind:</span>
		<span
			class="nodrag"
			contenteditable="true"
			role="textbox"
			oninput={handleKeyBindChange}>{data.keyBind ?? 'F6'}</span
		>
	</div>
</BaseNode>
