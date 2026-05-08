<script>
  import { getJson, deleteJson, ApiError } from '../lib/api.js';
  import { appState } from '../stores/app.svelte.js';
  import { formatRelative } from '../lib/relativeTime.js';
  import HistoryEntryDetailModal from './HistoryEntryDetailModal.svelte';
  import ConfirmDeleteModal from './ConfirmDeleteModal.svelte';
  import InlineError from './InlineError.svelte';
  import Spinner from './Spinner.svelte';
  import Toast from './Toast.svelte';

  let entries = $state([]);
  let loading = $state(true);
  let errorVisible = $state(false);
  let errorMessage = $state('');

  let detailOpen = $state(false);
  let detailEntryId = $state('');
  let detailFilename = $state('');

  let confirmOpen = $state(false);
  let confirmEntry = $state(null);

  let toastVisible = $state(false);
  let toastMessage = $state('');

  // Recargar al montar y cada vez que TabCifrar bumpea historyVersion (D-15 refresh).
  $effect(() => {
    appState.historyVersion;
    void loadList();
  });

  async function loadList() {
    loading = true;
    errorVisible = false;
    errorMessage = '';
    try {
      const resp = await getJson('/api/history');
      entries = (resp.entries ?? []).slice().sort((a, b) => b.timestamp.localeCompare(a.timestamp));
    } catch (e) {
      errorMessage = e instanceof ApiError ? e.message : 'No se pudo conectar al servidor local.';
      errorVisible = true;
    } finally {
      loading = false;
    }
  }

  function openDetail(entry) {
    detailEntryId = entry.id;
    detailFilename = entry.filename;
    detailOpen = true;
  }

  function openConfirm(entry) {
    confirmEntry = entry;
    confirmOpen = true;
  }

  async function handleDelete() {
    if (!confirmEntry) return;
    const id = confirmEntry.id;
    try {
      await deleteJson(`/api/history/${id}`);
      entries = entries.filter((e) => e.id !== id);
      toastMessage = 'Entrada borrada';
      toastVisible = true;
    } catch (e) {
      errorMessage = e instanceof ApiError ? e.message : 'No se pudo borrar la entrada.';
      errorVisible = true;
    }
  }
</script>

<div class="tab-historial">
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  {#if loading}
    <p class="loading"><Spinner /> Cargando…</p>
  {:else if entries.length === 0}
    <div class="empty">
      <h2 class="empty-title">Sin backups cifrados aún</h2>
      <p class="empty-body">Cifra un descriptor con el modo histórico activo para que aparezca aquí.</p>
    </div>
  {:else}
    <ul class="entries" aria-label="Lista de backups cifrados">
      {#each entries as entry (entry.id)}
        <li class="entry">
          <div class="info">
            <span class="when" title={entry.timestamp}>{formatRelative(entry.timestamp)}</span>
            <span class="filename">{entry.filename}</span>
            <span class="size">{Math.round(entry.size_bytes / 1024 * 10) / 10} KB</span>
          </div>
          <div class="actions">
            <button type="button" class="btn btn-secondary" onclick={() => openDetail(entry)}>Ver</button>
            <button type="button" class="btn btn-destructive" onclick={() => openConfirm(entry)}>Borrar</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<HistoryEntryDetailModal bind:open={detailOpen} entryId={detailEntryId} filename={detailFilename} />
<ConfirmDeleteModal bind:open={confirmOpen} entry={confirmEntry} onConfirm={handleDelete} />
<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .tab-historial { display: flex; flex-direction: column; gap: var(--space-md); }
  .loading { font-size: var(--font-size-label); color: var(--color-text-secondary); display: flex; align-items: center; gap: var(--space-sm); }
  .empty { text-align: center; padding: var(--space-2xl) var(--space-md); }
  .empty-title { margin: 0 0 var(--space-sm) 0; font-size: var(--font-size-heading); font-weight: var(--font-weight-bold); color: var(--color-text-primary); }
  .empty-body { margin: 0; font-size: var(--font-size-body); color: var(--color-text-secondary); line-height: var(--line-height-body); }
  .entries { list-style: none; padding: 0; margin: 0; background: var(--color-surface-raised); border: 1px solid var(--color-border); border-radius: var(--radius-card); overflow: hidden; }
  .entry { display: flex; align-items: center; justify-content: space-between; padding: var(--space-sm-plus) var(--space-md); min-height: 56px; border-bottom: 1px solid var(--color-border); flex-wrap: wrap; gap: var(--space-md); }
  .entry:last-child { border-bottom: 0; }
  .info { display: flex; flex-direction: column; gap: 2px; flex: 1; min-width: 0; }
  .when { font-size: var(--font-size-label); color: var(--color-text-primary); }
  .filename { font-family: var(--font-mono); font-size: var(--font-size-mono); color: var(--color-text-secondary); word-break: break-all; }
  .size { font-size: var(--font-size-label); color: var(--color-text-secondary); }
  .actions { display: flex; gap: var(--space-sm); }
  .btn { min-height: var(--touch-target); min-width: var(--touch-target); padding: var(--space-sm) var(--space-md); border-radius: var(--radius-button); font-size: var(--font-size-label); cursor: pointer; transition: background-color var(--transition-color); }
  .btn-secondary { background: transparent; color: var(--color-text-primary); border: 1px solid var(--color-border); }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
  .btn-destructive { background: transparent; color: var(--color-destructive); border: 1px solid var(--color-destructive); }
  .btn-destructive:hover { background: var(--color-destructive); color: var(--color-destructive-fg); }
</style>
