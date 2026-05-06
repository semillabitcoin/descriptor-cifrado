<script>
  let {
    open = $bindable(false),
    title = '',
    children,
    onConfirm,
    onCancel,
    confirmLabel = 'Aceptar',
    cancelLabel = 'Cancelar',
    confirmVariant = 'primary',  // 'primary' | 'destructive'
    confirmLoading = false,
  } = $props();

  let panel;
  let cancelBtn;

  $effect(() => {
    if (open) {
      // Focus default → cancelBtn (D-36)
      queueMicrotask(() => cancelBtn?.focus());
      function onKey(e) {
        if (e.key === 'Escape') { e.preventDefault(); cancel(); }
        if (e.key === 'Tab') trapFocus(e);
      }
      document.addEventListener('keydown', onKey);
      return () => document.removeEventListener('keydown', onKey);
    }
  });

  function trapFocus(e) {
    if (!panel) return;
    const focusables = panel.querySelectorAll('button, [href], input, [tabindex]:not([tabindex="-1"])');
    if (!focusables.length) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (e.shiftKey && document.activeElement === first) {
      e.preventDefault(); last.focus();
    } else if (!e.shiftKey && document.activeElement === last) {
      e.preventDefault(); first.focus();
    }
  }

  function cancel() {
    open = false;
    onCancel?.();
  }
  function confirm() {
    onConfirm?.();
  }
</script>

{#if open}
  <div class="backdrop" onclick={cancel} role="presentation"></div>
  <div
    class="panel"
    bind:this={panel}
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
  >
    <h2 id="modal-title" class="title">{title}</h2>
    <div class="body">
      {@render children?.()}
    </div>
    <div class="actions">
      <button
        type="button"
        class="btn btn-secondary"
        bind:this={cancelBtn}
        onclick={cancel}
        disabled={confirmLoading}
      >{cancelLabel}</button>
      <button
        type="button"
        class="btn btn-{confirmVariant}"
        onclick={confirm}
        disabled={confirmLoading}
      >{confirmLabel}</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(2px);
    z-index: 9000;
  }
  .panel {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    background: var(--color-surface-raised);
    color: var(--color-text-primary);
    border-radius: var(--radius-modal);
    padding: var(--space-lg);
    max-width: 400px;
    width: calc(100% - var(--space-xl));
    box-shadow: var(--shadow-modal);
    z-index: 9001;
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
  }
  .body {
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
    margin-bottom: var(--space-lg);
  }
  .actions {
    display: flex; gap: var(--space-sm); justify-content: flex-end;
  }
  .btn {
    min-height: var(--touch-target);
    min-width: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color), color var(--transition-color);
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
  .btn-destructive {
    background: var(--color-destructive);
    color: var(--color-destructive-fg);
    border: 0;
  }
  .btn-destructive:hover:not(:disabled) { background: var(--color-destructive-hover); }
</style>
