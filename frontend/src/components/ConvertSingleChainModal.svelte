<script>
  let { open = $bindable(false), convertedDescriptor = '', onConfirm = () => {}, onCancel = () => {} } = $props();

  let cancelButton = $state(null);

  $effect(() => {
    if (open && cancelButton) {
      queueMicrotask(() => cancelButton.focus());
    }
  });

  function handleKeydown(e) {
    if (e.key === 'Escape') {
      handleCancel();
    }
  }

  function handleCancel() {
    open = false;
    onCancel();
  }

  function handleConfirm() {
    open = false;
    onConfirm();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
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
      aria-labelledby="convert-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="convert-title" class="title">Descriptor receive-only detectado</h2>
      <div class="body">
        <p class="description">
          El descriptor usa derivación single-chain <code>/N/*</code> (típico de Nunchuk Desktop).
          Para cifrar de forma compatible con BIP 380 multipath, se convertirá a
          <code>/&lt;base;base+1&gt;/*</code> en cada cosigner y se eliminará el checksum original
          (que ya no sería válido tras la conversión).
        </p>
        <p class="bip-note">
          Convención BIP44/BIP389: el par receive/change se aparea automáticamente según la regla
          par-impar. <code>/0/*</code> y <code>/1/*</code> ambos producen <code>&lt;0;1&gt;/*</code>;
          <code>/2/*</code> y <code>/3/*</code> producen <code>&lt;2;3&gt;/*</code>.
        </p>
        <p class="preview-label">Resultado tras la conversión:</p>
        <pre class="preview">{convertedDescriptor}</pre>
      </div>
      <div class="actions">
        <button bind:this={cancelButton} type="button" class="btn btn-secondary" onclick={handleCancel}>Cancelar</button>
        <button type="button" class="btn btn-primary" onclick={handleConfirm}>Convertir y cifrar</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(2px);
    z-index: 9000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }
  .panel {
    background: var(--color-surface-raised);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    max-width: 480px;
    width: 100%;
    box-shadow: var(--shadow-modal);
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
  }
  .body {
    margin-bottom: var(--space-lg);
  }
  .description {
    margin: 0;
    font-size: var(--font-size-body);
    color: var(--color-text-primary);
    line-height: var(--line-height-body);
  }
  .description code {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    background: var(--color-surface-sunken);
    padding: 1px 4px;
    border-radius: 4px;
  }
  .bip-note {
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    margin: var(--space-sm) 0 0 0;
  }
  .bip-note code {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    background: var(--color-surface-sunken);
    padding: 1px 4px;
    border-radius: 4px;
  }
  .preview-label {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    margin: var(--space-md) 0 0 0;
  }
  .preview {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    padding: var(--space-sm) var(--space-md);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    margin-top: var(--space-sm);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
  .btn {
    min-height: var(--touch-target);
    min-width: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    cursor: pointer;
    transition: background-color var(--transition-color);
  }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover:not(:disabled) { background: var(--color-surface-sunken); }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }
</style>
