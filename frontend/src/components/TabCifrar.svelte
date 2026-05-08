<script>
  import { postJson, ApiError } from '../lib/api.js';
  import { appState, bumpHistoryVersion } from '../stores/app.svelte.js';
  import { detectSingleChain, convertSingleChainToMultipath } from '../lib/descriptor.js';
  import InlineError from './InlineError.svelte';
  import CifrarOutputs from './CifrarOutputs.svelte';
  import ConvertSingleChainModal from './ConvertSingleChainModal.svelte';
  import Toast from './Toast.svelte';
  import Spinner from './Spinner.svelte';

  let descriptor = $state('');
  let label = $state('');
  let result = $state(null);
  let resultLabel = $state('');
  let loading = $state(false);
  let errorVisible = $state(false);
  let errorMessage = $state('');
  let warningToast = $state(false);
  let warningMessage = $state('');
  let descriptorDragOver = $state(false);
  let descriptorFilename = $state('');
  let descriptorFileInput;
  let singleChainModalOpen = $state(false);
  let singleChainConverted = $state('');

  const LABEL_PATTERN = /^[a-zA-Z0-9 _-]+$/;
  const labelTrimmed = $derived(label.trim());
  const labelValid = $derived(
    labelTrimmed.length > 0 && labelTrimmed.length <= 80 && LABEL_PATTERN.test(labelTrimmed)
  );

  const PLACEHOLDER =
    'wsh(or_d(pk([fp/48h/0h/0h/2h]xpub.../<0;1>/*),and_v(v:pkh([fp/48h/0h/0h/2h]xpub.../<2;3>/*),older(N))))#checksum';

  function handleDescriptorDragOver(e) {
    e.preventDefault();
    descriptorDragOver = true;
  }
  function handleDescriptorDragLeave() {
    descriptorDragOver = false;
  }
  async function handleDescriptorDrop(e) {
    e.preventDefault();
    descriptorDragOver = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) await loadDescriptorFile(file);
  }
  async function handleDescriptorFilePick(e) {
    const file = e.target?.files?.[0];
    if (!file) return;
    await loadDescriptorFile(file);
    if (descriptorFileInput) descriptorFileInput.value = '';
  }
  async function loadDescriptorFile(file) {
    try {
      const text = await file.text();
      descriptor = text.trim();
      descriptorFilename = file.name;
    } catch {
      errorMessage = 'No se pudo leer el archivo de descriptor.';
      errorVisible = true;
    }
  }

  function handleLimpiar() {
    descriptor = '';
    label = '';
    result = null;
    resultLabel = '';
    loading = false;
    errorVisible = false;
    errorMessage = '';
    warningToast = false;
    warningMessage = '';
    descriptorFilename = '';
    descriptorDragOver = false;
    if (descriptorFileInput) descriptorFileInput.value = '';
  }

  async function handleCifrar() {
    if (!descriptor.trim() || !labelValid || loading) return;

    // Interceptar descriptores single-chain ANTES de lanzar el POST
    if (detectSingleChain(descriptor.trim())) {
      singleChainConverted = convertSingleChainToMultipath(descriptor.trim());
      singleChainModalOpen = true;
      return; // esperar confirmación del modal
    }

    loading = true;
    errorVisible = false;
    errorMessage = '';
    result = null;
    resultLabel = '';
    try {
      const resp = await postJson('/api/encrypt', { descriptor: descriptor.trim() });
      result = resp;
      resultLabel = labelTrimmed;
      // D-12: si historyEnabled ON, persistir tras éxito (fire-and-warn).
      if (appState.historyEnabled) {
        try {
          await postJson('/api/history', { bed_b64: resp.bed_b64, label: labelTrimmed });
          bumpHistoryVersion();
        } catch {
          warningMessage = 'Cifrado OK, pero no se guardó en historial';
          warningToast = true;
        }
      }
    } catch (e) {
      if (e instanceof ApiError) {
        // QR_TOO_LARGE (D-11): añadir nota extra (UI-SPEC §Error States)
        if (e.code === 'QR_TOO_LARGE') {
          errorMessage = `${e.message} Usa el archivo .bed o el texto armored.`;
        } else {
          errorMessage = e.message;
        }
      } else {
        errorMessage = 'No se pudo conectar al servidor local.';
      }
      errorVisible = true;
    } finally {
      loading = false;
    }
  }

  function handleSubmit(e) {
    e.preventDefault();
    handleCifrar();
  }

  function handleSingleChainConfirm() {
    descriptor = singleChainConverted;  // actualizar textarea con valor convertido
    singleChainModalOpen = false;
    // Lanzar handleCifrar de nuevo — el descriptor ya es multipath, no volverá a interceptarse
    handleCifrar();
  }

  function handleSingleChainCancel() {
    singleChainModalOpen = false;
    // No hacer nada; el descriptor queda sin cambios
  }
</script>

<form class="form" onsubmit={handleSubmit} novalidate>
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  <div class="field">
    <label for="label-input" class="label">Nombre</label>
    <input
      id="label-input"
      class="input"
      type="text"
      bind:value={label}
      maxlength="80"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      spellcheck="false"
      aria-describedby="label-help"
    />
    <p id="label-help" class="help">
      Se usará como nombre del archivo descargado y en el historial. Letras, números, espacios, guiones y guiones bajos.
    </p>
  </div>

  <div class="field">
    <label for="descriptor-input" class="label">Descriptor</label>

    <!-- Dropzone para cargar desde archivo -->
    <div
      class="dropzone"
      class:dragover={descriptorDragOver}
      role="button"
      tabindex="0"
      aria-label="Soltar archivo con descriptor aquí o pulsa para seleccionar"
      ondragover={handleDescriptorDragOver}
      ondragleave={handleDescriptorDragLeave}
      ondrop={handleDescriptorDrop}
      onclick={() => descriptorFileInput?.click()}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); descriptorFileInput?.click(); } }}
    >
      {#if descriptorFilename}
        <p class="dropzone-text">
          Archivo cargado: <strong class="filename">{descriptorFilename}</strong>
        </p>
        <button type="button" class="btn btn-ghost" onclick={(e) => { e.stopPropagation(); descriptorFilename = ''; descriptor = ''; if (descriptorFileInput) descriptorFileInput.value = ''; }}>
          Cambiar archivo
        </button>
      {:else}
        <p class="dropzone-text">Arrastra el archivo con descriptor aquí o pulsa para seleccionar</p>
      {/if}
      <input
        bind:this={descriptorFileInput}
        type="file"
        accept=".txt,.descriptor,.json,.jsonl,text/plain,application/json"
        hidden
        onchange={handleDescriptorFilePick}
      />
    </div>

    <p class="separator">— o pega el descriptor —</p>

    <textarea
      id="descriptor-input"
      class="textarea"
      bind:value={descriptor}
      placeholder={PLACEHOLDER}
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      aria-describedby="descriptor-help"
      rows="6"
    ></textarea>
    <p id="descriptor-help" class="help">
      Nada se envía a internet. El cifrado y descifrado son locales.
    </p>
  </div>

  <div class="btn-row">
    <button type="submit" class="btn btn-primary" disabled={!descriptor.trim() || !labelValid || loading}>
      {#if loading}
        <Spinner /> <span>Cifrando…</span>
      {:else}
        <span>Cifrar</span>
      {/if}
    </button>
    <button
      type="button"
      class="btn btn-ghost"
      disabled={!descriptor && result === null && !errorVisible && !warningToast}
      onclick={handleLimpiar}
    >
      Limpiar
    </button>
  </div>
</form>

{#if result}
  <CifrarOutputs {result} label={resultLabel} />
{/if}

<Toast bind:visible={warningToast} message={warningMessage} />

<ConvertSingleChainModal
  bind:open={singleChainModalOpen}
  convertedDescriptor={singleChainConverted}
  onConfirm={handleSingleChainConfirm}
  onCancel={handleSingleChainCancel}
/>

<style>
  .form { display: flex; flex-direction: column; gap: var(--space-md); }
  .field { display: flex; flex-direction: column; gap: var(--space-sm); }
  .label {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    line-height: var(--line-height-label);
    color: var(--color-text-primary);
  }
  .textarea {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    line-height: var(--line-height-mono);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    background: var(--color-surface-sunken);
    color: var(--color-text-primary);
    padding: var(--space-sm) var(--space-md);
    min-height: 120px;
    resize: vertical;
    width: 100%;
    transition: border-color var(--transition-color), box-shadow var(--transition-focus);
  }
  .textarea:focus {
    outline: 0;
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-focus);
  }
  .input {
    font-family: inherit;
    font-size: var(--font-size-body);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-input);
    background: var(--color-surface-sunken);
    color: var(--color-text-primary);
    padding: var(--space-sm) var(--space-md);
    width: 100%;
    transition: border-color var(--transition-color), box-shadow var(--transition-focus);
  }
  .input:focus {
    outline: 0;
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-focus);
  }
  .help {
    margin: 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
  }
  .help code {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    background: var(--color-surface-sunken);
    padding: 1px 4px;
    border-radius: 4px;
  }
  .btn {
    min-height: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-regular);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    transition: background-color var(--transition-color), opacity var(--transition-disabled);
  }
  .btn:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-primary {
    background: var(--color-accent);
    color: var(--color-accent-fg);
    border: 0;
  }
  .btn-primary:hover:not(:disabled) { background: var(--color-accent-hover); }
  .btn-row {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }
  .dropzone {
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-card);
    background: var(--color-surface-sunken);
    padding: var(--space-xl) var(--space-md);
    text-align: center;
    cursor: pointer;
    transition: border-color var(--transition-color), background-color var(--transition-color);
  }
  .dropzone:hover, .dropzone:focus-visible {
    border-color: var(--color-border-focus);
    outline: 0;
  }
  .dropzone.dragover {
    border-color: var(--color-accent);
    background: var(--color-surface-raised);
  }
  .dropzone-text {
    margin: 0 0 var(--space-sm) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
  }
  .filename {
    font-family: var(--font-mono);
    font-size: var(--font-size-mono);
    color: var(--color-text-primary);
  }
  .separator {
    text-align: center;
    margin: var(--space-sm) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
  }
  .btn-ghost {
    background: transparent;
    color: var(--color-text-secondary);
    border: 0;
    align-self: flex-start;
  }
  .btn-ghost:hover { background: var(--color-surface-sunken); color: var(--color-text-primary); }
</style>
