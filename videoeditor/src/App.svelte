<script lang="ts">
  import PaneLayout from '$lib/components/PaneLayout.svelte';
</script>

<div class="app">
  <PaneLayout />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-rows: 1fr;
  }
</style>
