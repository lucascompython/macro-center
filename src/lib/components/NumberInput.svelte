<script lang="ts">
	interface Props {
		value: number;
		intOnly?: boolean;
		maxWidth?: string;
	}

	let { value = $bindable(0), intOnly = false, maxWidth = '140px' }: Props = $props();

	function handleKeyPress(e: KeyboardEvent & { currentTarget: EventTarget & HTMLSpanElement }) {
		const isDot = e.key === '.';
		const alreadyHasDot = e.currentTarget.innerText.includes('.');
		const isNotNumber = isNaN(parseFloat(e.key)) && e.key !== '.';

		if ((isDot && (intOnly || alreadyHasDot)) || isNotNumber) {
			e.preventDefault();
		}
	}

	function handleInput(e: Event & { currentTarget: EventTarget & HTMLSpanElement }) {
		const parsed = parseFloat(e.currentTarget.innerText);
		if (!isNaN(parsed)) {
			value = parsed;
		}
	}
</script>

<span
	class="nodrag number-input"
	contenteditable="true"
	role="textbox"
	tabindex="0"
	style:max-width={maxWidth}
	onkeypress={handleKeyPress}
	oninput={handleInput}>{value}</span
>

<style>
	.number-input {
		background: none;
		outline: none;
		border: none;
		color: white;
		cursor: text;
		border-radius: 0.25rem;
		background: #4c4d4f;
		min-width: 3rem;
		padding: 0.1rem 0.25rem;
		margin-left: 0.5rem;
		border: 1px solid transparent;
		transition: border 0.2s ease;
		font-size: 0.9rem;
		font-weight: 400;
	}

	.number-input:focus {
		border: 1px solid #e92a67;
	}
</style>
