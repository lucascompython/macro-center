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

	function handleKeyBindChange(e: Event & { currentTarget: HTMLInputElement }) {
		updateNodeData(id, { shortcut: e.currentTarget.value });
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
		<input class="nodrag" type="text" value={data.shortcut ?? 'F6'} oninput={handleKeyBindChange} />
	</div>
</BaseNode>
