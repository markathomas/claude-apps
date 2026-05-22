<script lang="ts">
  import type { RecentProject } from '$lib/types';

  interface Props {
    open: boolean;
    items: RecentProject[];
    onPick: (path: string) => void;
    onCancel: () => void;
  }
  let { open, items, onPick, onCancel }: Props = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" role="presentation" onclick={onCancel}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus a11y_no_noninteractive_tabindex -->
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="recent-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <h2 id="recent-title">Recent projects</h2>
      {#if items.length === 0}
        <p class="empty">No recent projects.</p>
      {:else}
        <ul>
          {#each items as item (item.path)}
            <li>
              <button type="button" onclick={() => onPick(item.path)}>
                <span class="name">{item.name}</span>
                <span class="path">{item.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
      <div class="actions">
        <button type="button" onclick={onCancel}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: grid; place-items: center; }
  .modal { background: #1f1f1f; padding: 1.25rem; border-radius: 8px; min-width: 480px; max-width: 720px; }
  h2 { margin: 0 0 0.75rem; font-size: 1rem; }
  .empty { opacity: 0.6; }
  ul { list-style: none; padding: 0; margin: 0 0 1rem; max-height: 50vh; overflow: auto; }
  li button {
    display: grid;
    grid-template-columns: 1fr;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 0;
    color: inherit;
    padding: 0.5rem 0.75rem;
    border-radius: 4px;
    cursor: pointer;
  }
  li button:hover { background: #2a2a2a; }
  .name { font-weight: 600; }
  .path { font-size: 0.8rem; opacity: 0.6; }
  .actions { display: flex; justify-content: flex-end; }
  .actions button { padding: 0.4rem 0.9rem; border-radius: 4px; border: 1px solid #555; background: #2a2a2a; color: inherit; cursor: pointer; }
</style>
