<script>
  import { appState, setTheme } from '../stores/app.svelte.js';

  function cycle() {
    const next = appState.theme === 'auto' ? 'light' : appState.theme === 'light' ? 'dark' : 'auto';
    setTheme(next);
  }

  const labels = { light: 'Tema claro', dark: 'Tema oscuro', auto: 'Tema automático' };
  const icons = { light: '☀', dark: '☾', auto: '◐' };
</script>

<button
  type="button"
  class="theme-toggle"
  aria-label={labels[appState.theme]}
  title={labels[appState.theme]}
  onclick={cycle}
>
  <span aria-hidden="true">{icons[appState.theme]}</span>
</button>

<style>
  .theme-toggle {
    background: transparent;
    border: 1px solid var(--color-border);
    color: var(--color-text-primary);
    border-radius: var(--radius-button);
    min-width: var(--touch-target);
    min-height: var(--touch-target);
    padding: var(--space-sm);
    cursor: pointer;
    font-size: 18px;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .theme-toggle:hover { background: var(--color-surface-sunken); }
</style>
