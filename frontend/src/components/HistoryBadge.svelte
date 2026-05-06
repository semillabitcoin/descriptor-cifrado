<script>
  import { appState } from '../stores/app.svelte.js';
</script>

{#if appState.historyEnabled}
  <span class="badge" aria-label="Modo histórico activo">Modo histórico activo</span>
{/if}

<style>
  .badge {
    display: inline-block;
    padding: var(--space-xs) var(--space-sm);
    background: var(--color-history-badge-bg);
    color: var(--color-history-badge-text);
    border-radius: var(--radius-pill);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: 1;
  }
</style>
