<script lang="ts">
  import MediaPool from './MediaPool.svelte';
  import PreviewPlayer from './PreviewPlayer.svelte';
  import Inspector from './Inspector.svelte';
  import Timeline from './Timeline.svelte';
</script>

<div class="layout">
  <div class="top">
    <MediaPool />
    <PreviewPlayer />
  </div>
  <Inspector />
  <Timeline />
</div>

<style>
  .layout {
    display: grid;
    grid-template-rows: 1fr auto auto;
    height: 100%;
    min-height: 0;
  }
  .top {
    display: grid;
    grid-template-columns: 280px 1fr;
    min-height: 0;
    border-bottom: 1px solid #2a2a2a;
  }
  .top > :global(.pane:first-child) {
    border-right: 1px solid #2a2a2a;
  }
  :global(.layout > .pane) {
    border-bottom: 1px solid #2a2a2a;
  }
</style>
