<script lang="ts">
  import BaseNode from "./BaseNode.svelte";
  import type { BreakLoopNodeData } from "$lib/types";

  interface Props {
    id: string;
    data: BreakLoopNodeData;
  }

  let { id, data }: Props = $props();
</script>

<BaseNode
  {id}
  data={{
    title: data.title ?? "Break",
    subline: data.subline ?? "Exit the nearest loop",
    enableSource: false,
  }}
/>
