<script>
  import { copyToClipboard } from '../lib/clipboard.js';
  import { triggerDownloadText } from '../lib/download.js';
  import AnimatedQrModal from './AnimatedQrModal.svelte';
  import Toast from './Toast.svelte';

  let { descriptor, composedDescriptor = null } = $props();

  let copyLabel = $state('Copiar al portapapeles');
  let copyComposedLabel = $state('Copiar al portapapeles');
  let copyResetTimer;
  let copyComposedResetTimer;
  let toastVisible = $state(false);
  let toastMessage = $state('');
  let qrOpen = $state(false);

  // Rama Liana/clásica: isJson detecta si es JSON para elegir extensión de descarga.
  // En rama Sparrow composedDescriptor está presente y se usa la sección secundaria.
  const isJson = $derived(!composedDescriptor && descriptor.trimStart().startsWith('{'));
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

  async function handleCopyComposed() {
    const ok = await copyToClipboard(composedDescriptor);
    if (ok) {
      copyComposedLabel = 'Copiado ✓';
      showToast('Copiado al portapapeles');
      clearTimeout(copyComposedResetTimer);
      copyComposedResetTimer = setTimeout(() => { copyComposedLabel = 'Copiar al portapapeles'; }, 1500);
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

  function handleDownloadJsonl() {
    const ts = new Date().toISOString().replace(/[:.]/g, '-');
    triggerDownloadText(descriptor, `sparrow-labels-${ts}.jsonl`, 'text/plain');
  }

  function handleShowQr() {
    qrOpen = true;
  }
</script>

{#if composedDescriptor}
  <!-- Rama Sparrow BIP329: mostrar descriptor compuesto como sección primaria
       y JSONL completo como sección secundaria. Sin botón QR (JSONL grande). -->

  <div class="result-zone" aria-label="Descriptor recuperado">
    <h2 class="title">Descriptor recuperado</h2>
    <p class="subtitle">Descriptor canónico listo para reimportar en Sparrow.</p>

    <pre class="descriptor" aria-label="Descriptor multisig compuesto">{composedDescriptor}</pre>

    <div class="actions">
      <button type="button" class="btn btn-primary" onclick={handleCopyComposed}>{copyComposedLabel}</button>
    </div>
  </div>

  <div class="result-zone secondary" aria-label="Backup completo JSONL">
    <h2 class="title">Backup completo (JSONL)</h2>
    <p class="subtitle">Archivo de etiquetas BIP329 exportado por Sparrow. Impórtalo en Sparrow tras crear la wallet con el descriptor de arriba.</p>

    <pre class="descriptor" aria-label="JSONL BIP329 completo">{descriptor}</pre>

    <div class="actions">
      <button type="button" class="btn btn-secondary" onclick={handleDownloadJsonl}>Descargar .jsonl</button>
    </div>
  </div>

{:else}
  <!-- Rama Liana / descriptor clásico: layout original sin cambios -->

  <div class="result-zone" aria-label="Descriptor recuperado">
    <h2 class="title">Descriptor recuperado</h2>

    <pre class="descriptor" aria-label="Descriptor multisig en claro">{descriptor}</pre>

    <div class="actions">
      <button type="button" class="btn btn-primary" onclick={handleCopy}>{copyLabel}</button>
      <button type="button" class="btn btn-secondary" onclick={handleDownload}>{downloadLabel}</button>
      <button type="button" class="btn btn-secondary" onclick={handleShowQr}>Mostrar QR</button>
    </div>
  </div>

  <AnimatedQrModal bind:open={qrOpen} text={descriptor} />
{/if}

<Toast bind:visible={toastVisible} message={toastMessage} />

<style>
  .result-zone {
    margin-top: var(--space-xl);
    background: var(--color-surface-raised);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    box-shadow: var(--shadow-card);
  }
  .result-zone.secondary {
    margin-top: var(--space-md);
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-heading);
    color: var(--color-text-primary);
  }
  .subtitle {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
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
