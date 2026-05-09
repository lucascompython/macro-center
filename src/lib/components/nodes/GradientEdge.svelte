<script lang="ts">
	import { BaseEdge, getBezierPath, type EdgeProps } from '@xyflow/svelte';

	let {
		id,
		sourceX,
		sourceY,
		sourcePosition,
		targetX,
		targetY,
		targetPosition,
		markerEnd,
		data
	}: EdgeProps = $props();

	const edgeStyle = $derived(
		data?.kind === 'value'
			? 'stroke: #38d0ff; stroke-width: 2; stroke-dasharray: 5 4; stroke-opacity: 0.9'
			: undefined
	);

	// Calculate bezier path with small hack for straight lines to show gradient
	let edgePath = $derived.by(() => {
		const xEqual = sourceX === targetX;
		const yEqual = sourceY === targetY;
		const [path] = getBezierPath({
			sourceX: xEqual ? sourceX + 0.0001 : sourceX,
			sourceY: yEqual ? sourceY + 0.0001 : sourceY,
			sourcePosition,
			targetX,
			targetY,
			targetPosition
		});
		return path;
	});
</script>

<BaseEdge {id} path={edgePath} {markerEnd} style={edgeStyle} />
