<script lang="ts">
  import MenuBar from '$lib/components/MenuBar.svelte';
  import PaneLayout from '$lib/components/PaneLayout.svelte';
</script>

<div class="app">
  <MenuBar />
  <PaneLayout />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-rows: auto 1fr;
  }
</style>
