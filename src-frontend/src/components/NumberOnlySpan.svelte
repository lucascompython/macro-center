<script lang="ts">
  export let value = 0;
  export let maxWidth = "140";
  export let intOnly = false;

  function handleKeyPress(
    e: KeyboardEvent & {
      currentTarget: EventTarget & HTMLSpanElement;
    }
  ): boolean {
    const isDot = e.key === ".";
    const alreadyHasDot = e.currentTarget.innerText.includes(".");
    const isNotNumber = isNaN(parseFloat(e.key));

    if ((isDot && (intOnly || alreadyHasDot)) || isNotNumber) {
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
  on:keypress={handleKeyPress}
  on:input={handleInput}>{value}</span
>
