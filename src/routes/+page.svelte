<script lang="ts">
    import Flow from "$lib/components/Flow.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
</script>

<svelte:head>
    <title>Macro Center</title>
    <meta
        name="description"
        content="Visual Programming Tool for keyboard and mouse macros"
    />
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link
        rel="preconnect"
        href="https://fonts.gstatic.com"
        crossorigin="anonymous"
    />
    <link
        href="https://fonts.googleapis.com/css2?family=Fira+Mono:wght@400;500;700&family=Inter:wght@400;500;600;700&display=swap"
        rel="stylesheet"
    />
</svelte:head>

<main>
    <div class="layout">
        <Sidebar />
        <div class="flow-wrapper">
            <Flow />
        </div>
    </div>
</main>

<style>
    :global(body) {
        margin: 0;
        padding: 0;
        overflow: hidden;
        font-family: "Inter", "Fira Mono", sans-serif;
        background-color: #121212;
    }

    main {
        width: 100vw;
        height: 100vh;
        overflow: hidden;
    }

    .layout {
        display: flex;
        width: 100%;
        height: 100%;
    }

    .flow-wrapper {
        flex: 1;
        position: relative;
        height: 100%;
    }
</style>
