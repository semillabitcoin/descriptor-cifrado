<script>
  import { postMultipart, ApiError } from '../lib/api.js';
  import { validateXpub } from '../lib/xpub.js';
  import InlineError from './InlineError.svelte';
  import DescifrarOutputs from './DescifrarOutputs.svelte';
  import Spinner from './Spinner.svelte';

  // Estado LOCAL — al desmontar (cambio de tab) Svelte lo descarta automáticamente (D-16, D-17).
  let bedFile = $state(null);          // File object si vino de drop o file picker .bed
  let bedFilename = $state('');        // nombre visible del archivo cargado
  let armoredText = $state('');        // textarea armored
  let xpubText = $state('');
  let descriptor = $state(null);       // resultado descifrado — vive solo aquí
  let loading = $state(false);
  let errorVisible = $state(false);
  let errorMessage = $state('');
  let dragOver = $state(false);
  let bedFileInput;
  let xpubFileInput;

  // El botón Descifrar está disabled hasta que tenemos:
  // (a) un .bed (file binario) O un texto armored no vacío
  // (b) una xpub que matchea el regex
  let bedReady = $derived(bedFile !== null || armoredText.trim().length > 0);
  let xpubReady = $derived(validateXpub(xpubText));
  let canSubmit = $derived(bedReady && xpubReady && !loading);

  function handleClearResult() {
    descriptor = null;
  }

  function handleDragOver(e) {
    e.preventDefault();
    dragOver = true;
  }
  function handleDragLeave() {
    dragOver = false;
  }
  function handleDrop(e) {
    e.preventDefault();
    dragOver = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) {
      acceptBedFile(file);
    }
  }
  function handleBedFilePick(e) {
    const file = e.target?.files?.[0];
    if (file) acceptBedFile(file);
  }
  function acceptBedFile(file) {
    bedFile = file;
    bedFilename = file.name;
    // Si el usuario cargó un binario, vaciar el textarea armored para evitar confusión.
    armoredText = '';
  }
  function clearBedFile() {
    bedFile = null;
    bedFilename = '';
    if (bedFileInput) bedFileInput.value = '';
  }
  function handleArmoredInput() {
    // Si el usuario escribe armored, descartar el file binario para evitar enviar ambos.
    if (armoredText.trim().length > 0 && bedFile) {
      clearBedFile();
    }
  }
  async function handleXpubFilePick(e) {
    const file = e.target?.files?.[0];
    if (!file) return;
    try {
      const text = await file.text();
      xpubText = text.trim();
    } catch {
      errorMessage = 'No se pudo leer el archivo de xpub.';
      errorVisible = true;
    }
    if (xpubFileInput) xpubFileInput.value = '';
  }

  async function handleDescifrar() {
    if (!canSubmit) return;
    loading = true;
    errorVisible = false;
    errorMessage = '';
    descriptor = null;
    try {
      const formData = new FormData();
      if (bedFile) {
        formData.append('bed', bedFile, bedFile.name);
      } else {
        // Enviar armored como Blob text (el backend autodetecta por bytes mágicos).
        const armoredBlob = new Blob([armoredText.trim()], { type: 'text/plain' });
        formData.append('bed', armoredBlob, 'armored.txt');
      }
      formData.append('xpub', xpubText.trim());

      const resp = await postMultipart('/api/decrypt', formData);
      descriptor = resp.descriptor;
      // D-17: limpiar xpub tras descifrado exitoso (security default).
      xpubText = '';
    } catch (e) {
      if (e instanceof ApiError) {
        errorMessage = e.message;
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
    handleDescifrar();
  }
</script>

<form class="form" onsubmit={handleSubmit} novalidate>
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  <!-- Sección 1: .bed o armored -->
  <fieldset class="field">
    <legend class="legend">Backup cifrado (`.bed` o armored)</legend>

    <div
      class="dropzone"
      class:dragover={dragOver}
      role="button"
      tabindex="0"
      aria-label="Soltar archivo .bed aquí o pulsa para seleccionar"
      ondragover={handleDragOver}
      ondragleave={handleDragLeave}
      ondrop={handleDrop}
      onclick={() => bedFileInput?.click()}
      onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); bedFileInput?.click(); } }}
    >
      {#if bedFile}
        <p class="dropzone-text">
          Archivo cargado: <strong class="filename">{bedFilename}</strong>
        </p>
        <button type="button" class="btn btn-ghost" onclick={(e) => { e.stopPropagation(); clearBedFile(); }}>
          Cambiar archivo
        </button>
      {:else}
        <p class="dropzone-text">Arrastra el archivo `.bed` aquí o pulsa para seleccionar</p>
      {/if}
      <input
        bind:this={bedFileInput}
        type="file"
        accept=".bed,application/octet-stream"
        hidden
        onchange={handleBedFilePick}
      />
    </div>

    <p class="separator">— o pega el texto armored —</p>

    <label for="armored-input" class="label visually-hidden">Texto armored</label>
    <textarea
      id="armored-input"
      class="textarea"
      bind:value={armoredText}
      oninput={handleArmoredInput}
      placeholder="-----BEGIN BITCOIN ENCRYPTED BACKUP-----"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      rows="4"
      disabled={bedFile !== null}
    ></textarea>
  </fieldset>

  <!-- Sección 2: xpub -->
  <fieldset class="field">
    <legend class="legend">xpub cosigner</legend>

    <label for="xpub-input" class="label visually-hidden">xpub cosigner</label>
    <textarea
      id="xpub-input"
      class="textarea xpub"
      bind:value={xpubText}
      placeholder="xpub6... o tpub…"
      spellcheck="false"
      autocomplete="off"
      autocorrect="off"
      autocapitalize="off"
      rows="3"
      aria-describedby="xpub-help"
    ></textarea>
    <p id="xpub-help" class="help">
      Cualquier xpub cosigner del multisig sirve. La xpub se borra automáticamente tras descifrar.
    </p>

    <button type="button" class="btn btn-ghost" onclick={() => xpubFileInput?.click()}>
      Subir archivo con xpub
    </button>
    <input
      bind:this={xpubFileInput}
      type="file"
      accept=".txt,text/plain"
      hidden
      onchange={handleXpubFilePick}
    />
  </fieldset>

  <button type="submit" class="btn btn-primary" disabled={!canSubmit}>
    {#if loading}
      <Spinner /> <span>Descifrando…</span>
    {:else}
      <span>Descifrar</span>
    {/if}
  </button>
</form>

{#if descriptor}
  <div class="result-wrapper">
    <DescifrarOutputs {descriptor} />
    <button type="button" class="btn btn-ghost clear-btn" onclick={handleClearResult}>
      Limpiar resultado
    </button>
  </div>
{/if}

<style>
  .form { display: flex; flex-direction: column; gap: var(--space-lg); }
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
    border: 0;
    padding: 0;
    margin: 0;
  }
  .legend {
    font-size: var(--font-size-label);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
    margin-bottom: var(--space-sm);
    padding: 0;
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
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0,0,0,0);
    border: 0;
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
    min-height: 100px;
    resize: vertical;
    width: 100%;
    transition: border-color var(--transition-color), box-shadow var(--transition-focus);
  }
  .textarea.xpub { min-height: 72px; }
  .textarea:focus {
    outline: 0;
    border-color: var(--color-border-focus);
    box-shadow: var(--shadow-focus);
  }
  .textarea:disabled { opacity: 0.5; }
  .help {
    margin: 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
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
  .btn-ghost {
    background: transparent;
    color: var(--color-text-secondary);
    border: 0;
    align-self: flex-start;
  }
  .btn-ghost:hover { background: var(--color-surface-sunken); color: var(--color-text-primary); }
  .clear-btn { margin-top: var(--space-md); }
  .result-wrapper { display: flex; flex-direction: column; }
</style>
