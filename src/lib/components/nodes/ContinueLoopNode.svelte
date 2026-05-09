<script lang="ts">
	import BaseNode from './BaseNode.svelte';
	import type { ContinueLoopNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: ContinueLoopNodeData;
	}

	let { id, data }: Props = $props();
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Continue',
		subline: data.subline ?? 'Start the next iteration',
		enableSource: false
	}}
/>
