<script>
  import { postJson, ApiError } from '../lib/api.js';
  import { appState } from '../stores/app.svelte.js';
  import InlineError from './InlineError.svelte';
  import CifrarOutputs from './CifrarOutputs.svelte';
  import Toast from './Toast.svelte';
  import Spinner from './Spinner.svelte';

  let descriptor = $state('');
  let result = $state(null);
  let loading = $state(false);
  let errorVisible = $state(false);
  let errorMessage = $state('');
  let warningToast = $state(false);
  let warningMessage = $state('');

  const PLACEHOLDER =
    'wsh(multi(2,[fp/48h/0h/0h/2h]xpub…/<0;1>/*,[fp/48h/0h/0h/2h]xpub…/<0;1>/*))#checksum';

  async function handleCifrar() {
    if (!descriptor.trim() || loading) return;
    loading = true;
    errorVisible = false;
    errorMessage = '';
    result = null;
    try {
      const resp = await postJson('/api/encrypt', { descriptor: descriptor.trim() });
      result = resp;
      // D-12: si historyEnabled ON, persistir tras éxito (fire-and-warn).
      if (appState.historyEnabled) {
        try {
          await postJson('/api/history', { bed_b64: resp.bed_b64 });
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
</script>

<form class="form" onsubmit={handleSubmit} novalidate>
  <InlineError bind:visible={errorVisible} message={errorMessage} />

  <div class="field">
    <label for="descriptor-input" class="label">Descriptor multisig</label>
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
      Pega el descriptor con derivación <code>&lt;0;1&gt;/*</code>. Nada se envía a internet.
    </p>
  </div>

  <button type="submit" class="btn btn-primary" disabled={!descriptor.trim() || loading}>
    {#if loading}
      <Spinner /> <span>Cifrando…</span>
    {:else}
      <span>Cifrar</span>
    {/if}
  </button>
</form>

{#if result}
  <CifrarOutputs {result} />
{/if}

<Toast bind:visible={warningToast} message={warningMessage} />

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
</style>
