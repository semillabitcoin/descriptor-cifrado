<script>
  let { message = '', visible = $bindable(false), durationMs = 3000 } = $props();

  let timer;
  $effect(() => {
    if (visible) {
      clearTimeout(timer);
      timer = setTimeout(() => { visible = false; }, durationMs);
    }
    return () => clearTimeout(timer);
  });
</script>

{#if visible}
  <div class="toast" role="status" aria-live="polite">{message}</div>
{/if}

<style>
  .toast {
    position: fixed;
    top: var(--space-md);
    right: var(--space-md);
    width: 320px;
    padding: var(--space-sm-plus) var(--space-md);
    border-radius: var(--radius-toast);
    background: var(--color-toast-bg);
    color: var(--color-toast-text);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: var(--line-height-label);
    box-shadow: var(--shadow-modal);
    z-index: 9999;
    animation: slide-in var(--transition-toast-in);
  }
  @keyframes slide-in {
    from { transform: translateX(120%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  @media (prefers-reduced-motion: reduce) {
    .toast { animation: none; }
  }
</style>
