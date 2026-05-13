<script lang="ts">
  import { NodeResizer } from "@xyflow/svelte";
  import { getContext } from "svelte";
  import { SUBFLOW_RENAME_CONTEXT, type RenameSubflow } from "$lib/editor-context";
  import type { BaseNodeData } from "$lib/types";

  interface Props {
    selected?: boolean;
    data: BaseNodeData & {
      definitionId?: string;
    };
  }

  let { data, selected = false }: Props = $props();
  let titleInput: HTMLInputElement | undefined = $state();
  let draftName = $state("Subflow");

  const renameSubflow = getContext<RenameSubflow | undefined>(SUBFLOW_RENAME_CONTEXT);

  $effect(() => {
    const title = data.title ?? "Subflow";
    if (
      typeof document !== "undefined" &&
      document.activeElement !== titleInput &&
      title !== draftName
    ) {
      draftName = title;
    }
  });

  function rename(value: string) {
    if (!data.definitionId) return;
    renameSubflow?.(data.definitionId, value);
  }

  function handleTitleInput(event: Event & { currentTarget: HTMLInputElement }) {
    draftName = event.currentTarget.value;
    rename(draftName);
  }

  function handleTitleBlur() {
    const nextName = draftName.trim() || "Subflow";
    draftName = nextName;
    rename(nextName);
  }
</script>

<NodeResizer
  isVisible={selected}
  minWidth={420}
  minHeight={260}
  color="#a853ba"
  handleClass="subflow-resize-handle"
  lineClass="subflow-resize-line"
/>

<div class="subflow-frame">
  <div class="subflow-gradient"></div>
  <div class="subflow-group">
    <div class="subflow-header drag-handle">
      <div>
        <input
          bind:this={titleInput}
          class="nodrag subflow-title-input"
          value={draftName}
          aria-label="Subflow name"
          placeholder="Subflow"
          oninput={handleTitleInput}
          onblur={handleTitleBlur}
        />
        <div class="subflow-subline">{data.subline ?? "Visible submacro definition"}</div>
      </div>
      <div class="badge">Subflow</div>
    </div>
  </div>
</div>

<style>
  .subflow-frame {
    border-radius: 10px;
    box-sizing: border-box;
    box-shadow:
      10px 0 15px rgba(42, 138, 246, 0.3),
      -10px 0 15px rgba(233, 42, 103, 0.3);
    height: 100%;
    min-height: 100%;
    min-width: 100%;
    overflow: hidden;
    padding: 2px;
    position: relative;
    width: 100%;
  }

  .subflow-gradient {
    background: conic-gradient(
      from -160deg at 50% 50%,
      #e92a67 0deg,
      #a853ba 120deg,
      #2a8af6 240deg,
      #e92a67 360deg
    );
    border-radius: 100%;
    content: "";
    left: 50%;
    padding-bottom: calc(100% * 1.41421356237);
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: calc(100% * 1.41421356237);
  }

  .subflow-group {
    background: rgba(17, 17, 17, 0.92);
    border-radius: 8px;
    box-sizing: border-box;
    height: 100%;
    min-height: 100%;
    min-width: 100%;
    overflow: hidden;
    position: relative;
    width: 100%;
    z-index: 1;
  }

  .subflow-header {
    align-items: center;
    background: rgba(12, 14, 18, 0.74);
    border-bottom: 1px solid rgba(168, 83, 186, 0.34);
    display: flex;
    height: 44px;
    justify-content: space-between;
    padding: 0 0.85rem;
  }

  .subflow-title-input {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 4px;
    color: #f2f2f2;
    font-family: "Inter", sans-serif;
    font-size: 0.95rem;
    font-weight: 650;
    line-height: 1;
    margin: 0;
    max-width: 240px;
    min-width: 8rem;
    outline: none;
    padding: 0.12rem 0.2rem;
    transition:
      background 0.2s ease,
      border 0.2s ease;
  }

  .subflow-title-input:focus {
    background: rgba(168, 83, 186, 0.16);
    border-color: rgba(168, 83, 186, 0.62);
  }

  .subflow-subline {
    color: #8fa1ad;
    font-family: "Inter", sans-serif;
    font-size: 0.72rem;
    margin-top: 0.24rem;
  }

  .badge {
    background: rgba(168, 83, 186, 0.12);
    border: 1px solid rgba(168, 83, 186, 0.46);
    border-radius: 4px;
    color: #cfafff;
    font-family: "Inter", sans-serif;
    font-size: 0.68rem;
    padding: 0.18rem 0.38rem;
    text-transform: uppercase;
  }

  :global(.subflow-resize-handle) {
    background: #111 !important;
    border: 1px solid #a853ba !important;
    border-radius: 3px !important;
    height: 8px !important;
    width: 8px !important;
  }

  :global(.subflow-resize-line) {
    border-color: rgba(168, 83, 186, 0.65) !important;
  }
</style>
