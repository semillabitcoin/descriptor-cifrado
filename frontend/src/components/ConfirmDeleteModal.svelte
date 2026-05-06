<script>
  let { open = $bindable(false), entry = null, onConfirm = () => {} } = $props();

  let cancelButton = $state(null);
  let loading = $state(false);

  $effect(() => {
    if (open && cancelButton) {
      // Default focus en Cancelar (D-36) tras un microtask para que el DOM esté pintado.
      queueMicrotask(() => cancelButton.focus());
    }
  });

  function handleKeydown(e) {
    if (e.key === 'Escape' && !loading) {
      open = false;
    }
  }

  async function handleConfirm() {
    loading = true;
    try {
      await onConfirm();
      open = false;
    } finally {
      loading = false;
    }
  }

  function handleCancel() {
    if (!loading) open = false;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open && entry}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={handleCancel} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_interactive_supports_focus -->
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="confirm-title" class="title">Borrar backup cifrado</h2>
      <p class="body">
        ¿Eliminar el backup <code class="entry-id">{entry.timestamp} · {entry.id}</code>?
        Esta acción no se puede deshacer.
      </p>
      <div class="actions">
        <button bind:this={cancelButton} type="button" class="btn btn-secondary" onclick={handleCancel} disabled={loading}>Cancelar</button>
        <button type="button" class="btn btn-destructive" onclick={handleConfirm} disabled={loading}>
          {loading ? 'Borrando…' : 'Borrar'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.4); backdrop-filter: blur(2px); z-index: 9000; display: flex; align-items: center; justify-content: center; padding: var(--space-md); }
  .panel { background: var(--color-surface-raised); border-radius: var(--radius-card); padding: var(--space-lg); max-width: 400px; width: 100%; box-shadow: var(--shadow-modal); }
  .title { margin: 0 0 var(--space-md) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .body { margin: 0 0 var(--space-lg) 0; font-size: var(--font-size-body); color: var(--color-text-primary); line-height: var(--line-height-body); }
  .entry-id { font-family: var(--font-mono); font-size: var(--font-size-mono); background: var(--color-surface-sunken); padding: 1px 4px; border-radius: 4px; }
  .actions { display: flex; justify-content: flex-end; gap: var(--space-sm); }
  .btn { min-height: var(--touch-target); min-width: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover:not(:disabled) { background: var(--color-surface-sunken); }
  .btn-destructive { background: var(--color-destructive); color: var(--color-destructive-fg); border: 0; }
  .btn-destructive:hover:not(:disabled) { background: var(--color-destructive-hover); }
</style>
