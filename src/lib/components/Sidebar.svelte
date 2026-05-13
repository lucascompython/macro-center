<script lang="ts">
	import {
		nodeTemplateCategories,
		nodeTemplateCategoryClass,
		nodeTemplatesForCategory,
		type NodeTemplate,
	} from "$lib/node-palette";

	let collapsed = $state(false);

	function onDragStart(event: DragEvent, template: NodeTemplate) {
		if (!event.dataTransfer) return;

		const payload = JSON.stringify({ type: template.type, data: template.data });
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
			{#each nodeTemplateCategories as category}
				<div class="category">
					<h3>{category}</h3>
					{#each nodeTemplatesForCategory(category) as template (template.type)}
						<button
							class={`dndnode ${nodeTemplateCategoryClass(template.category)}`}
							draggable="true"
							ondragstart={(event) => onDragStart(event, template)}
						>
							{template.label}
						</button>
					{/each}
				</div>
			{/each}
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
