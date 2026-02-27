<script lang="ts">
	import { useSvelteFlow } from "@xyflow/svelte";
	import BaseNode from "./BaseNode.svelte";
	import type { KeyBindNodeData } from "$lib/types";

	interface Props {
		id: string;
		data: KeyBindNodeData;
	}

	let { id, data }: Props = $props();

	const { updateNodeData } = useSvelteFlow();

	let recording = $state(false);
	let display = $state(data.shortcut ?? "");

	const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

	// map KeyboardEvent.key values to the accelerator format tauri expects
	function keyToAccelerator(key: string): string {
		const map: Record<string, string> = {
			Control: "Ctrl",
			Meta: "Super",
			" ": "Space",
			ArrowUp: "Up",
			ArrowDown: "Down",
			ArrowLeft: "Left",
			ArrowRight: "Right",
			Escape: "Escape",
			Enter: "Enter",
			Backspace: "Backspace",
			Delete: "Delete",
			Tab: "Tab",
			Home: "Home",
			End: "End",
			PageUp: "PageUp",
			PageDown: "PageDown",
			Insert: "Insert",
		};
		if (map[key]) return map[key];
		// F-keys
		if (/^F\d{1,2}$/.test(key)) return key;
		// single character keys - uppercase for the accelerator
		if (key.length === 1) return key.toUpperCase();
		return key;
	}

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
			const parts: string[] = [];
			if (e.ctrlKey) parts.push("Ctrl");
			if (e.shiftKey) parts.push("Shift");
			if (e.altKey) parts.push("Alt");
			if (e.metaKey) parts.push("Super");
			display = parts.join("+") + "+...";
			return;
		}

		// non-modifier key pressed - build the full shortcut and commit
		const parts: string[] = [];
		if (e.ctrlKey) parts.push("Ctrl");
		if (e.shiftKey) parts.push("Shift");
		if (e.altKey) parts.push("Alt");
		if (e.metaKey) parts.push("Super");
		parts.push(keyToAccelerator(e.key));

		stopRecording(parts.join("+"));
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
