<script>
  let { message = '', visible = $bindable(false) } = $props();
  function close() { visible = false; }
</script>

{#if visible && message}
  <div class="alert" role="alert">
    <span class="icon" aria-hidden="true">⚠</span>
    <span class="message">{message}</span>
    <button type="button" class="close" aria-label="Cerrar mensaje de error" onclick={close}>×</button>
  </div>
{/if}

<style>
  .alert {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    padding: var(--space-sm-plus) var(--space-md);
    background: var(--color-warning-bg);
    border-left: 4px solid var(--color-warning-border);
    border-radius: var(--radius-card);
    color: var(--color-warning-text);
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
  }
  .icon { font-size: 18px; line-height: 1; }
  .message { flex: 1; }
  .close {
    background: transparent;
    border: 0;
    color: inherit;
    font-size: 20px;
    cursor: pointer;
    padding: 0 var(--space-xs);
    min-width: var(--touch-target);
    min-height: var(--touch-target);
  }
  .close:hover { background: rgba(0,0,0,0.06); border-radius: var(--radius-button); }
</style>
