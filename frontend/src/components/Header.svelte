<script>
  import HistoryBadge from './HistoryBadge.svelte';
  import HistoryToggle from './HistoryToggle.svelte';
  import ThemeToggle from './ThemeToggle.svelte';
</script>

<header class="header">
  <h1 class="logo">BED</h1>
  <div class="spacer"></div>
  <HistoryBadge />
  <HistoryToggle />
  <ThemeToggle />
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    height: 56px;
    padding: 0 var(--space-md);
    background: var(--color-surface-raised);
    border-bottom: 1px solid var(--color-border);
    box-shadow: var(--shadow-header);
    position: sticky;
    top: 0;
    z-index: 100;
  }
  .logo {
    margin: 0;
    font-size: var(--font-size-display);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-display);
    color: var(--color-text-primary);
    letter-spacing: -0.02em;
  }
  .spacer { flex: 1; }
  @media (max-width: 540px) {
    .header { gap: var(--space-sm); padding: 0 var(--space-sm); }
    .logo { font-size: 20px; }
  }
</style>
