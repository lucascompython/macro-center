<script lang="ts">
	import { 
		ActionMode, 
		CoordinateMode, 
		Axis, 
		MouseButton 
	} from '$lib/types';

	function onDragStart(event: DragEvent, nodeType: string, data: any = {}) {
		if (!event.dataTransfer) return;
		
		const payload = JSON.stringify({ type: nodeType, data });
		event.dataTransfer.setData('application/svelteflow', payload);
		event.dataTransfer.effectAllowed = 'move';
	}
</script>

<aside class="sidebar">
	<div class="header">
		<h2>Components</h2>
		<p class="subtitle">Drag to add</p>
	</div>

	<div class="category">
		<h3>Triggers</h3>
		<button
			class="dndnode trigger"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'keyBindNode', { title: 'Key Bind', subline: 'Trigger macro' })}
		>
			Key Bind
		</button>
	</div>

	<div class="category">
		<h3>Actions</h3>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'typeNode', { title: 'Type Text', text: 'Hello' })}
		>
			Type Text
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'keyNode', { title: 'Key Press', mode: ActionMode.CLICK })}
		>
			Key Press
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'mousePressNode', { title: 'Mouse Click', button: MouseButton.LEFT })}
		>
			Mouse Click
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'mouseMoveNode', { title: 'Move Mouse', x: 0, y: 0 })}
		>
			Move Mouse
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'scrollMouseNode', { title: 'Scroll Mouse', amount: 3 })}
		>
			Scroll Mouse
		</button>
	</div>

	<div class="category">
		<h3>Logic</h3>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'delayNode', { title: 'Delay', delay: 1000 })}
		>
			Delay
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) => onDragStart(event, 'conditionalNode', { title: 'Conditional', condition: 'true' })}
		>
			Conditional
		</button>
	</div>
</aside>

<style>
	.sidebar {
		background-color: #1a1a1a;
		border-right: 1px solid #333;
		display: flex;
		flex-direction: column;
		height: 100%;
		padding: 1rem;
		width: 250px;
		font-family: 'Inter', sans-serif;
		overflow-y: auto;
	}

	.header {
		margin-bottom: 1.5rem;
	}

	h2 {
		color: #fff;
		font-size: 1.1rem;
		margin: 0;
		font-weight: 600;
	}

	.subtitle {
		color: #777;
		font-size: 0.8rem;
		margin: 0.25rem 0 0 0;
	}

	.category {
		margin-bottom: 1.5rem;
	}

	h3 {
		color: #777;
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0 0 0.5rem 0.25rem;
		font-weight: 500;
	}

	.dndnode {
		background: #2c2d2f;
		border: 1px solid #3e3e3e;
		border-radius: 6px;
		color: #e0e0e0;
		cursor: grab;
		font-size: 0.9rem;
		margin-bottom: 0.5rem;
		padding: 0.75rem 1rem;
		transition: all 0.2s ease;
		display: flex;
		align-items: center;
	}

	.dndnode:hover {
		background: #38393c;
		border-color: #555;
		transform: translateY(-1px);
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.dndnode::before {
		content: '';
		display: block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		margin-right: 10px;
	}

	.dndnode.trigger::before {
		background: #2a8af6;
		box-shadow: 0 0 8px rgba(42, 138, 246, 0.4);
	}

	.dndnode.action::before {
		background: #a853ba;
		box-shadow: 0 0 8px rgba(168, 83, 186, 0.4);
	}

	.dndnode.logic::before {
		background: #e92a67;
		box-shadow: 0 0 8px rgba(233, 42, 103, 0.4);
	}
</style>
