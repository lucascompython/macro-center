<script lang="ts">
	import { Handle, Position, useSvelteFlow } from '@xyflow/svelte';
	import BaseNode from './BaseNode.svelte';
	import ConditionEditor from '$lib/components/ConditionEditor.svelte';
	import { defaultConditionExpression } from '$lib/logic';
	import { valueHandle } from '$lib/graph';
	import type { CompareNodeData, ConditionExpression } from '$lib/types';

	interface Props {
		id: string;
		data: CompareNodeData;
	}

	let { id, data }: Props = $props();
	const { updateNodeData } = useSvelteFlow();

	function handleConditionChange(conditionExpression: ConditionExpression) {
		updateNodeData(id, { conditionExpression });
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? 'Compare',
		subline: data.subline ?? 'Outputs true or false',
		enableTarget: false,
		enableSource: false
	}}
>
	<ConditionEditor
		nodeId={id}
		expression={data.conditionExpression ?? defaultConditionExpression}
		onChange={handleConditionChange}
	/>
</BaseNode>

<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('left')}
	style="left: 34%; border-color: #38d0ff; background: #102a36"
/>
<Handle
	type="target"
	position={Position.Top}
	id={valueHandle('right')}
	style="left: 66%; border-color: #38d0ff; background: #102a36"
/>
<Handle
	type="source"
	position={Position.Bottom}
	id={valueHandle('value')}
	style="left: 50%; border-color: #38d0ff; background: #102a36"
/>
