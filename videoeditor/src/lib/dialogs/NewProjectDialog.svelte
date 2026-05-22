<script lang="ts">
  interface Props {
    open: boolean;
    onCreate: (name: string) => void;
    onCancel: () => void;
  }
  let { open, onCreate, onCancel }: Props = $props();
  let name = $state('Untitled');

  function submit(e: Event) {
    e.preventDefault();
    if (name.trim().length === 0) return;
    onCreate(name.trim());
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" role="presentation" onclick={onCancel}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus a11y_no_noninteractive_tabindex -->
    <div class="modal" role="dialog" aria-modal="true" aria-labelledby="new-title" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <h2 id="new-title">New project</h2>
      <form onsubmit={submit}>
        <label>
          <span>Name</span>
          <!-- svelte-ignore a11y_autofocus -->
          <input bind:value={name} autofocus />
        </label>
        <div class="actions">
          <button type="button" onclick={onCancel}>Cancel</button>
          <button type="submit">Create</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: grid; place-items: center; }
  .modal { background: #1f1f1f; padding: 1.25rem; border-radius: 8px; min-width: 360px; }
  h2 { margin: 0 0 0.75rem; font-size: 1rem; }
  label { display: grid; gap: 0.25rem; margin-bottom: 1rem; }
  input { padding: 0.5rem; background: #2a2a2a; border: 1px solid #444; color: inherit; border-radius: 4px; }
  .actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
  button { padding: 0.4rem 0.9rem; border-radius: 4px; border: 1px solid #555; background: #2a2a2a; color: inherit; cursor: pointer; }
  button[type="submit"] { background: #2563eb; border-color: #2563eb; }
</style>
