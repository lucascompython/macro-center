<script lang="ts">
	import { ActionMode, CoordinateMode, Axis, MouseButton } from "$lib/types";
	import { defaultConditionExpression } from "$lib/logic";

	let collapsed = $state(false);

	function onDragStart(event: DragEvent, nodeType: string, data: any = {}) {
		if (!event.dataTransfer) return;

		const payload = JSON.stringify({ type: nodeType, data });
		event.dataTransfer.setData("application/svelteflow", payload);
		event.dataTransfer.effectAllowed = "move";
	}
</script>

<aside class="sidebar" class:collapsed>
	<div class="header">
		{#if !collapsed}
			<div>
				<h2>Components</h2>
				<p class="subtitle">Drag to add</p>
			</div>
		{/if}
		<button
			class="collapse-btn"
			aria-label={collapsed ? "Expand components sidebar" : "Collapse components sidebar"}
			title={collapsed ? "Expand components" : "Collapse components"}
			onclick={() => (collapsed = !collapsed)}
		>
			{collapsed ? ">" : "<"}
		</button>
	</div>

	{#if !collapsed}
		<div class="content">
			<div class="category">
				<h3>Triggers</h3>
				<button
					class="dndnode trigger"
					draggable="true"
					ondragstart={(event) =>
						onDragStart(event, "keyBindNode", {
							title: "Key Bind",
							subline: "Trigger macro",
						})}
				>
					Key Bind
				</button>
			</div>

			<div class="category">
		<h3>Actions</h3>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "typeNode", {
					title: "Type Text",
					text: "Hello",
				})}
		>
			Type Text
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "keyNode", {
					title: "Key Press",
					mode: ActionMode.CLICK,
				})}
		>
			Key Press
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "mousePressNode", {
					title: "Mouse Click",
					button: MouseButton.LEFT,
				})}
		>
			Mouse Click
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "mouseMoveNode", {
					title: "Move Mouse",
					x: 0,
					y: 0,
				})}
		>
			Move Mouse
		</button>
		<button
			class="dndnode action"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "scrollMouseNode", {
					title: "Scroll Mouse",
					amount: 3,
				})}
		>
			Scroll Mouse
		</button>
	</div>

	<div class="category">
		<h3>Logic</h3>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "delayNode", {
					title: "Delay",
					delay: 1000,
				})}
		>
			Delay
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "conditionalNode", {
					title: "If",
					condition: "true",
					conditionExpression: defaultConditionExpression,
				})}
		>
			If
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "repeatLoopNode", {
					title: "Repeat",
					iterations: 3,
					indexVariable: "index",
				})}
		>
			Repeat
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "forEachLoopNode", {
					title: "For Each",
					items: [],
					itemVariable: "item",
					indexVariable: "index",
				})}
		>
			For Each
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "whileLoopNode", {
					title: "While",
					conditionExpression: defaultConditionExpression,
					indexVariable: "index",
					maxIterations: 100,
				})}
		>
			While
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) => onDragStart(event, "breakLoopNode", { title: "Break" })}
		>
			Break
		</button>
		<button
			class="dndnode logic"
			draggable="true"
			ondragstart={(event) => onDragStart(event, "continueLoopNode", { title: "Continue" })}
		>
			Continue
		</button>
	</div>

	<div class="category">
		<h3>Variables</h3>
		<button
			class="dndnode variable"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "valueNode", {
					title: "Value",
					valueType: "text",
					value: "",
				})}
		>
			Value
		</button>
		<button
			class="dndnode variable"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "setVariableNode", {
					title: "Set Variable",
					variableName: "value",
					valueType: "number",
					value: 0,
					scope: "macro",
				})}
		>
			Set Variable
		</button>
		<button
			class="dndnode variable"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "getVariableNode", {
					title: "Get Variable",
					variableName: "value",
				})}
		>
			Get Variable
		</button>
		<button
			class="dndnode variable"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "updateVariableNode", {
					title: "Update Variable",
					variableName: "value",
					operation: "increment",
					value: 1,
				})}
		>
			Update Variable
		</button>
		<button
			class="dndnode variable"
			draggable="true"
			ondragstart={(event) =>
				onDragStart(event, "compareNode", {
					title: "Compare",
					conditionExpression: defaultConditionExpression,
				})}
		>
			Compare
		</button>
	</div>
		</div>
	{/if}
</aside>

<style>
	.sidebar {
		background-color: #1a1a1a;
		border-right: 1px solid #333;
		display: flex;
		flex-direction: column;
		height: 100%;
		box-sizing: border-box;
		padding: 1rem;
		width: 250px;
		font-family: "Inter", sans-serif;
		overflow-y: auto;
		scrollbar-color: #4a4a4a #171717;
		scrollbar-width: thin;
		transition:
			width 0.18s ease,
			padding 0.18s ease;
	}

	.sidebar::-webkit-scrollbar {
		width: 10px;
	}

	.sidebar::-webkit-scrollbar-track {
		background: #171717;
		border-left: 1px solid #242424;
	}

	.sidebar::-webkit-scrollbar-thumb {
		background: #3a3b3d;
		border: 2px solid #171717;
		border-radius: 999px;
	}

	.sidebar::-webkit-scrollbar-thumb:hover {
		background: #555;
	}

	.sidebar.collapsed {
		border-right: 0;
		overflow: visible;
		padding: 0;
		width: 0;
	}

	.header {
		align-items: center;
		display: flex;
		justify-content: space-between;
		margin-bottom: 1.5rem;
	}

	.sidebar.collapsed .header {
		left: 0.75rem;
		margin-bottom: 0;
		position: absolute;
		top: 0.75rem;
		z-index: 30;
	}

	.content {
		min-height: 0;
	}

	.collapse-btn {
		align-items: center;
		background: #242528;
		border: 1px solid #3e3e3e;
		border-radius: 4px;
		color: #d8d8d8;
		cursor: pointer;
		display: flex;
		flex: 0 0 28px;
		font: inherit;
		font-size: 0.9rem;
		height: 28px;
		justify-content: center;
		padding: 0;
		width: 28px;
	}

	.collapse-btn:hover {
		background: #303136;
		border-color: #555;
		color: #fff;
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
		width: 100%;
	}

	.dndnode:hover {
		background: #38393c;
		border-color: #555;
		transform: translateY(-1px);
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
	}

	.dndnode::before {
		content: "";
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

	.dndnode.variable::before {
		background: #38d0ff;
		box-shadow: 0 0 8px rgba(56, 208, 255, 0.4);
	}
</style>
