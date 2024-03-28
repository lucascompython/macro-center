<script lang="ts">
  import { type EdgeProps, getBezierPath } from "@xyflow/svelte";

  type $$Props = EdgeProps;

  export let id: $$Props["id"];
  export let markerEnd: $$Props["markerEnd"] = undefined;
  export let sourceX: $$Props["sourceX"];
  export let sourceY: $$Props["sourceY"];
  export let sourcePosition: $$Props["sourcePosition"];
  export let targetX: $$Props["targetX"];
  export let targetY: $$Props["targetY"];
  export let targetPosition: $$Props["targetPosition"];

  let edgePath: string | undefined;

  // #region Unused props that are here just for not having the "unknown prop" warning

  export let source: $$Props["source"];
  source;
  export let target: $$Props["target"];
  target;
  export let animated: $$Props["animated"] = undefined;
  animated;
  export let selected: $$Props["selected"] = undefined;
  selected;
  export let label: $$Props["label"] = undefined;
  label;
  export let labelStyle: $$Props["labelStyle"] = undefined;
  labelStyle;
  export let data: $$Props["data"] = undefined;
  data;
  export let style: $$Props["style"] = undefined;
  style;
  export let interactionWidth: $$Props["interactionWidth"] = undefined;
  interactionWidth;
  export let type: $$Props["type"];
  type;
  export let sourceHandleId: $$Props["sourceHandleId"] = undefined;
  sourceHandleId;
  export let targetHandleId: $$Props["targetHandleId"] = undefined;
  targetHandleId;
  export let markerStart: $$Props["markerStart"] = undefined;
  markerStart;

  // #endregion

  $: {
    const xEqual = sourceX === targetX;
    const yEqual = sourceY === targetY;
    [edgePath] = getBezierPath({
      // we need this little hack in order to display the gradient for a straight line
      sourceX: xEqual ? sourceX + 0.0001 : sourceX,
      sourceY: yEqual ? sourceY + 0.0001 : sourceY,
      sourcePosition,
      targetX,
      targetY,
      targetPosition,
    });
  }
</script>

<path {id} class="svelte-flow__edge-path" d={edgePath} marker-end={markerEnd} />
