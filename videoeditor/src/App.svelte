<script lang="ts">
  import { onMount } from 'svelte';
  import MenuBar from '$lib/components/MenuBar.svelte';
  import PaneLayout from '$lib/components/PaneLayout.svelte';
  import { mediaActions } from '$lib/stores/mediaStore';

  onMount(() => {
    void mediaActions.initialize();
  });
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
