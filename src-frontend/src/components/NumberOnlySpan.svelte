<script lang="ts">
  export let value = 0;
  export let maxWidth = "140";
  export let intOnly = false;

  function handleKeyPress(
    e: KeyboardEvent & {
      currentTarget: EventTarget & HTMLSpanElement;
    }
  ): boolean {
    if (e.key === "." && !intOnly) {
      if (e.currentTarget.innerText.includes(".")) {
        e.preventDefault();
        return false;
      }
      return true;
    }
    if (isNaN(parseFloat(e.key))) {
      e.preventDefault();
      return false;
    }

    return true;
  }
  function handleInput(
    e: Event & {
      currentTarget: EventTarget & HTMLSpanElement;
    }
  ): void {
    value = parseFloat(e.currentTarget.innerText);
  }
</script>

<span
  class="nodrag"
  contenteditable="true"
  role="textbox"
  tabindex="0"
  style="max-width: {maxWidth}px;"
  on:keypress={(e) => handleKeyPress(e)}
  on:input={handleInput}>{value}</span
>
