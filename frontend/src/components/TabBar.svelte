<script>
  import { appState, setActiveTab } from '../stores/app.svelte.js';

  const TABS = [
    { id: 'cifrar', label: 'Cifrar' },
    { id: 'descifrar', label: 'Descifrar' },
  ];

  let visibleTabs = $derived(
    appState.historyEnabled
      ? [...TABS, { id: 'historial', label: 'Historial' }]
      : TABS
  );

  function handleKey(e, idx) {
    const tabs = visibleTabs;
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      setActiveTab(tabs[(idx + 1) % tabs.length].id);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setActiveTab(tabs[(idx - 1 + tabs.length) % tabs.length].id);
    } else if (e.key === 'Home') {
      e.preventDefault();
      setActiveTab(tabs[0].id);
    } else if (e.key === 'End') {
      e.preventDefault();
      setActiveTab(tabs[tabs.length - 1].id);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
<nav class="tabbar" role="tablist" aria-label="Secciones principales">
  {#each visibleTabs as tab, idx (tab.id)}
    <button
      type="button"
      role="tab"
      id="tab-{tab.id}"
      class="tab"
      class:active={appState.activeTab === tab.id}
      aria-selected={appState.activeTab === tab.id}
      aria-controls="panel-{tab.id}"
      tabindex={appState.activeTab === tab.id ? 0 : -1}
      onclick={() => setActiveTab(tab.id)}
      onkeydown={(e) => handleKey(e, idx)}
    >
      {tab.label}
    </button>
  {/each}
</nav>

<style>
  .tabbar {
    display: flex;
    gap: 0;
    border-bottom: 1px solid var(--color-border);
    max-width: 640px;
    margin: 0 auto;
    width: 100%;
    padding: 0 var(--space-md);
  }
  .tab {
    background: transparent;
    border: 0;
    border-bottom: 2px solid transparent;
    color: var(--color-text-secondary);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    line-height: var(--line-height-label);
    padding: 0 var(--space-md);
    min-height: var(--touch-target);
    cursor: pointer;
    transition: border-color var(--transition-tab), color var(--transition-tab), font-weight var(--transition-tab);
  }
  .tab:hover { color: var(--color-text-primary); }
  .tab.active {
    color: var(--color-text-primary);
    border-bottom-color: var(--color-accent);
    font-weight: var(--font-weight-bold);
  }
</style>
