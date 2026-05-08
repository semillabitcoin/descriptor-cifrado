<script>
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadText } from '../lib/download.js';
  import AnimatedQrModal from './AnimatedQrModal.svelte';
  import Toast from './Toast.svelte';

  let { descriptor } = $props();

  let copyLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');
  let qrOpen = $state(false);

  const isJson = $derived(descriptor.trimStart().startsWith('{'));
  const downloadLabel = $derived(isJson ? 'Descargar .json' : 'Descargar .txt');

  function showToast(msg) {
    toastMessage = msg;
    toastVisible = true;
  }

  async function handleCopy() {
    const ok = await copyToClipboard(descriptor);
    if (ok) {
      copyLabel = 'Copiado ✓';
      showToast('Copiado al portapapeles');
      clearTimeout(copyResetTimer);
      copyResetTimer = setTimeout(() => { copyLabel = 'Copiar al portapapeles'; }, 1500);
    } else {
      showToast('No se pudo copiar al portapapeles');
    }
  }

  function handleDownload() {
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    if (isJson) {
      triggerDownloadText(descriptor, `liana-backup-${ts}.json`, 'application/json');
    } else {
      triggerDownloadText(descriptor, `descriptor-${ts}.txt`, 'text/plain');
    }
  }

  function handleShowQr() {
    qrOpen = true;
  }
</script>

<div class="result-zone" aria-label="Descriptor recuperado">
  <h2 class="title">Descriptor recuperado</h2>

  <pre class="descriptor" aria-label="Descriptor multisig en claro">{descriptor}</pre>

  <div class="actions">
    <button type="button" class="btn btn-primary" onclick={handleCopy}>{copyLabel}</button>
    <button type="button" class="btn btn-secondary" onclick={handleDownload}>{downloadLabel}</button>
    <button type="button" class="btn btn-secondary" onclick={handleShowQr}>Mostrar QR</button>
  </div>
</div>

<Toast bind:visible={toastVisible} message={toastMessage} />
<AnimatedQrModal bind:open={qrOpen} text={descriptor} />

<style>
  .result-zone {
    margin-top: var(--space-xl);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    box-shadow: var(--shadow-card);
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
    color: var(--color-text-primary);
  }
  .descriptor {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    line-height: var(--line-height-mono);
    background: var(--color-surface-sunken);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    padding: var(--space-md);
    white-space: pre-wrap;
    word-break: break-all;
    overflow-x: auto;
    max-height: 200px;
    overflow-y: auto;
    margin: 0 0 var(--space-md) 0;
    color: var(--color-text-primary);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-sm);
  }
  .btn {
    min-height: var(--touch-target);
    min-width: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover { background: var(--color-accent-hover); }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
