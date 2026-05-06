<script>
  import { appState, setHistoryEnabled } from '../stores/app.svelte.js';
</script>

<label class="wrap">
  <span class="text">Historial</span>
  <button
    type="button"
    role="switch"
    class="track"
    class:on={appState.historyEnabled}
    aria-checked={appState.historyEnabled}
    aria-label="Modo histórico"
    onclick={() => setHistoryEnabled(!appState.historyEnabled)}
  >
    <span class="thumb" aria-hidden="true"></span>
  </button>
</label>

<style>
  .wrap {
    display: inline-flex;
    align-items: center;
    gap: var(--space-sm);
    cursor: pointer;
  }
  .text {
    font-size: var(--font-size-label);
    color: var(--color-text-primary);
    line-height: var(--line-height-label);
  }
  .track {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: var(--radius-pill);
    background: var(--color-border);
    border: 0;
    cursor: pointer;
    padding: 0;
    transition: background-color var(--transition-toggle);
  }
  .track.on { background: var(--color-accent); }
  .thumb {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    transition: left var(--transition-toggle);
    box-shadow: 0 1px 2px rgba(0,0,0,0.2);
  }
  .track.on .thumb { left: 23px; }
</style>
