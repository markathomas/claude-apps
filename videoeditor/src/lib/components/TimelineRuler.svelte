<script lang="ts">
  interface Props {
    durationMs: number;
    pxPerSec: number;
  }

  const { durationMs, pxPerSec }: Props = $props();

  const totalSeconds = $derived(Math.max(10, Math.ceil(durationMs / 1000) + 5));
  const ticks = $derived(
    Array.from({ length: totalSeconds + 1 }, (_, i) => ({
      sec: i,
      major: i % 5 === 0,
    })),
  );
  const widthPx = $derived(totalSeconds * pxPerSec);

  function formatLabel(sec: number): string {
    const m = Math.floor(sec / 60);
    const s = sec % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<div class="ruler" style="width: {widthPx}px">
  {#each ticks as t (t.sec)}
    <div
      class="tick"
      class:major={t.major}
      style="left: {t.sec * pxPerSec}px"
    >
      {#if t.major}
        <span class="label">{formatLabel(t.sec)}</span>
      {/if}
    </div>
  {/each}
</div>

<style>
  .ruler {
    position: relative;
    height: 22px;
    background: #1a1a1a;
    border-bottom: 1px solid #2a2a2a;
    flex-shrink: 0;
    user-select: none;
  }
  .tick {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: #333;
  }
  .tick.major {
    background: #555;
  }
  .label {
    position: absolute;
    top: 2px;
    left: 4px;
    font-size: 0.65rem;
    opacity: 0.6;
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
</style>
