<script lang="ts">
  interface Props {
    value: number;
    intOnly?: boolean;
    maxWidth?: string;
  }

  let { value = $bindable(0), intOnly = false, maxWidth = "140px" }: Props = $props();

  function handleInput(e: Event & { currentTarget: HTMLInputElement }) {
    const parsed = parseFloat(e.currentTarget.value);
    if (!isNaN(parsed)) {
      value = parsed;
    }
  }

  function handleKeyPress(e: KeyboardEvent) {
    const isDot = e.key === ".";
    const alreadyHasDot = String(value).includes(".");
    const isNotNumber = isNaN(parseFloat(e.key)) && e.key !== ".";

    if ((isDot && (intOnly || alreadyHasDot)) || isNotNumber) {
      e.preventDefault();
    }
  }
</script>

<input
  class="nodrag number-input"
  type="text"
  inputmode="decimal"
  {value}
  style:max-width={maxWidth}
  onkeypress={handleKeyPress}
  oninput={handleInput}
/>

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
    font-family: inherit;
  }

  .number-input:focus {
    border: 1px solid #e92a67;
  }
</style>
