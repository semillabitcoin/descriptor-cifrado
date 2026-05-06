<script>
  import { onMount } from 'svelte';
  import { appState, initFromStorage } from './stores/app.svelte.js';
  import Header from './components/Header.svelte';
  import TabBar from './components/TabBar.svelte';
  import TabCifrar from './components/TabCifrar.svelte';
  import TabDescifrar from './components/TabDescifrar.svelte';
  import TabHistorial from './components/TabHistorial.svelte';

  onMount(() => {
    initFromStorage();
  });
</script>

<Header />
<main>
  <TabBar />
  <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
  <section
    role="tabpanel"
    id="panel-cifrar"
    aria-labelledby="tab-cifrar"
    class="panel"
    hidden={appState.activeTab !== 'cifrar'}
  >
    <TabCifrar />
  </section>
  <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
  <section
    role="tabpanel"
    id="panel-descifrar"
    aria-labelledby="tab-descifrar"
    class="panel"
    hidden={appState.activeTab !== 'descifrar'}
  >
    <TabDescifrar />
  </section>
  {#if appState.historyEnabled}
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <section
      role="tabpanel"
      id="panel-historial"
      aria-labelledby="tab-historial"
      class="panel"
      hidden={appState.activeTab !== 'historial'}
    >
      <TabHistorial />
    </section>
  {/if}
</main>

<style>
  main {
    min-height: calc(100vh - 56px);
    padding-bottom: var(--space-3xl);
  }
  .panel {
    max-width: 640px;
    margin: 0 auto;
    padding: var(--space-2xl) var(--space-md) var(--space-3xl);
  }
  @media (min-width: 1024px) {
    .panel { padding-left: 0; padding-right: 0; }
  }
</style>
