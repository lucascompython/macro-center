<script lang="ts">
    import { Handle, Position, type NodeProps } from "@xyflow/svelte";

    let { data }: NodeProps = $props();
</script>

<div class="cloud gradient">
    <div>Cloud</div>
</div>
<div class="wrapper gradient">
    <div class="inner">
        <div class="body">
            {#if data.icon}
                <div class="icon">File</div>
            {/if}
            <div>
                <div class="title">{data.title}</div>
                {#if data.subtitle}
                    <div class="subtitle">{data.subtitle}</div>
                {/if}
            </div>
        </div>
        <Handle type="target" position={Position.Left} />
        <Handle type="source" position={Position.Right} />
    </div>
</div>
