<script lang="ts">
	import { useSvelteFlow } from "@xyflow/svelte";
	import BaseNode from "./BaseNode.svelte";
	import {
		MODIFIER_KEYS,
		modifierShortcutPreview,
		shortcutFromKeyboardEvent,
	} from "$lib/shortcuts";
	import type { KeyBindNodeData } from "$lib/types";

	interface Props {
		id: string;
		data: KeyBindNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	let recording = $state(false);
	let display = $state("");

	$effect(() => {
		if (!recording) {
			display = data.shortcut ?? "";
		}
	});

	function startRecording() {
		recording = true;
		display = "...";
	}

	function stopRecording(shortcut: string) {
		recording = false;
		display = shortcut;
		updateNodeData(id, { shortcut });
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (!recording) return;

		e.preventDefault();
		e.stopPropagation();

		// if only a modifier was pressed, show it live but don't commit
		if (MODIFIER_KEYS.has(e.key)) {
			display = modifierShortcutPreview(e);
			return;
		}

		// non-modifier key pressed - build the full shortcut and commit
		const shortcut = shortcutFromKeyboardEvent(e);
		if (shortcut) {
			stopRecording(shortcut);
		}
	}

	function handleBlur() {
		if (recording) {
			// cancelled - revert to previous value
			recording = false;
			display = data.shortcut ?? "";
		}
	}
</script>

<BaseNode
	{id}
	data={{
		title: data.title ?? "Key Bind",
		subline: data.subline ?? "Trigger macro with key combination",
		enableTarget: false,
		enableSource: true,
	}}
>
	<div class="keybind-row">
		<button
			class="nodrag keybind-btn"
			class:recording
			onkeydown={handleKeyDown}
			onclick={startRecording}
			onblur={handleBlur}
		>
			{#if recording}
				<span class="recording-dot"></span>
			{/if}
			<span class="keybind-label">{display || "Click to record"}</span>
		</button>
	</div>
</BaseNode>

<style>
	.keybind-row {
		padding: 0.35rem 0.5rem;
	}

	.keybind-btn {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.35rem 0.6rem;
		border-radius: 0.25rem;
		border: 1px solid #555;
		background: #3a3b3d;
		color: #e0e0e0;
		font-family: "Fira Mono", monospace;
		font-size: 0.85rem;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: left;
	}

	.keybind-btn:hover {
		border-color: #888;
		background: #444546;
	}

	.keybind-btn:focus {
		outline: none;
		border-color: #e92a67;
		box-shadow: 0 0 0 1px rgba(233, 42, 103, 0.3);
	}

	.keybind-btn.recording {
		border-color: #e92a67;
		background: #3a2029;
		animation: pulse-border 1.5s ease-in-out infinite;
	}

	@keyframes pulse-border {
		0%,
		100% {
			border-color: #e92a67;
		}
		50% {
			border-color: #ff4d8a;
		}
	}

	.recording-dot {
		display: block;
		flex: 0 0 6px !important;
		width: 6px !important;
		height: 6px !important;
		min-width: 6px !important;
		max-width: 6px !important;
		min-height: 6px !important;
		max-height: 6px !important;
		padding: 0 !important;
		margin: 0 !important;
		border-radius: 50% !important;
		background: #e92a67 !important;
		animation: blink 1s ease-in-out infinite;
	}

	@keyframes blink {
		0%,
		100% {
			opacity: 1;
		}
		50% {
			opacity: 0.3;
		}
	}

	.keybind-label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
