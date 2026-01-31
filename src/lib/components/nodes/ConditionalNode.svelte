<script lang="ts">
	import { Handle, Position, useSvelteFlow, useConnection } from '@xyflow/svelte';
	import SettingsIcon from '$lib/components/icons/SettingsIcon.svelte';
	import type { ConditionalNodeData } from '$lib/types';

	interface Props {
		id: string;
		data: ConditionalNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();
	const connection = useConnection();

	// Connection state - cast to compatible type to avoid TS issues
	const connectionState = $derived(connection.current as { 
		inProgress: boolean; 
		startHandle?: { nodeId?: string; type?: string } 
	});
	
	const isTarget = $derived(
		connectionState.inProgress &&
			connectionState.startHandle?.nodeId !== id &&
			connectionState.startHandle?.type === 'source'
	);

	function handleConditionChange(e: Event & { currentTarget: HTMLSpanElement }) {
		updateNodeData(id, { condition: e.currentTarget.innerText });
	}

	function handleSettingsClick() {
		console.log('Settings clicked for node:', id);
	}
</script>

<div class="base-node-container">
	<div class="wrapper gradient">
		<div class="inner">
			<div class="top-part">
				<div class="title">
					{data.title ?? 'Conditional'}
					<div class="subline">{data.subline ?? 'Branch based on condition'}</div>
				</div>

				<div class="settings-icon">
					<button onclick={handleSettingsClick}>
						<SettingsIcon />
					</button>
				</div>
			</div>
			<div class="content">
				<div class="input-container">
					<span>Condition:</span>
					<span class="nodrag" contenteditable="true" role="textbox" oninput={handleConditionChange}
						>{data.condition ?? 'value > 0'}</span
					>
				</div>

				<div class="outputs">
					<div class="output-label true-label">True</div>
					<div class="output-label false-label">False</div>
				</div>
			</div>
		</div>
	</div>

	<!-- Target handle (left) -->
	<Handle
		type="target"
		position={Position.Left}
		style="border-color: {isTarget ? '#e92a67' : ''}"
	/>

	<!-- True output handle (right top) -->
	<Handle
		type="source"
		position={Position.Right}
		id="true"
		style="top: 40%; border-color: #27d209"
	/>

	<!-- False output handle (right bottom) -->
	<Handle
		type="source"
		position={Position.Right}
		id="false"
		style="top: 70%; border-color: #ff6060"
	/>
</div>

<style>
	.base-node-container {
		position: relative;
	}

	.top-part {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.settings-icon {
		margin-left: 3rem;
	}

	.settings-icon button {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
	}

	.subline {
		font-size: 12px;
		color: #777;
		margin-top: 0.35rem;
		margin-bottom: 0.5rem;
	}

	.content {
		background: #2c2d2f;
		width: 100%;
		min-height: 0.5rem;
		cursor: default;
		border-radius: 0.25rem;
		padding: 0.25rem 0;
	}

	.outputs {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		padding-right: 0.5rem;
		margin-top: 0.5rem;
		gap: 0.75rem;
	}

	.output-label {
		font-size: 0.75rem;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.true-label {
		color: #27d209;
	}

	.false-label {
		color: #ff6060;
	}
</style>
