<script>
  let { open = $bindable(false), text = '' } = $props();

  let frames = $state([]); // strings BBQR (o un solo string si es QR estático)
  let frameImages = $state([]); // dataURLs renderizadas
  let currentFrame = $state(0);
  let loading = $state(false);
  let errorMessage = $state('');
  let isAnimated = $state(false);
  let intervalId;

  // Threshold: QR alfanumérico ECC-L cabe ~700 chars; bajamos a 500 por seguridad
  // (descriptors usan caracteres mixed-case que no caben en modo alfanumérico → modo binario más restrictivo).
  const STATIC_QR_THRESHOLD = 500;

  $effect(() => {
    if (open && text) {
      loadAndRender();
    } else {
      stopAnimation();
      frames = [];
      frameImages = [];
      currentFrame = 0;
      errorMessage = '';
    }
  });

  async function loadAndRender() {
    loading = true;
    errorMessage = '';
    try {
      const QRCode = (await import('qrcode')).default;
      if (text.length <= STATIC_QR_THRESHOLD) {
        // QR único estático
        isAnimated = false;
        const dataUrl = await QRCode.toDataURL(text, {
          errorCorrectionLevel: 'L',
          margin: 2,
          width: 300,
        });
        frameImages = [dataUrl];
        currentFrame = 0;
      } else {
        // BBQR animado
        isAnimated = true;
        const { splitQRs } = await import('bbqr');
        const encoder = new TextEncoder();
        const bytes = encoder.encode(text);
        // 'U' = utf-8 string; encoding 'Z' (zlib) lo comprime; 'H' es máximo
        const result = splitQRs(bytes, 'U', { encoding: 'Z', minSplit: 1, maxSplit: 99, minVersion: 5, maxVersion: 40 });
        frames = result.parts; // array de strings BBQR
        const renderedFrames = [];
        for (const part of frames) {
          const url = await QRCode.toDataURL(part, {
            errorCorrectionLevel: 'L',
            margin: 2,
            width: 300,
          });
          renderedFrames.push(url);
        }
        frameImages = renderedFrames;
        currentFrame = 0;
        startAnimation();
      }
    } catch (e) {
      errorMessage = 'No se pudo generar el código QR. Usa "Copiar al portapapeles" o "Descargar .txt".';
    } finally {
      loading = false;
    }
  }

  function startAnimation() {
    stopAnimation();
    if (frameImages.length > 1) {
      intervalId = setInterval(() => {
        currentFrame = (currentFrame + 1) % frameImages.length;
      }, 600);
    }
  }

  function stopAnimation() {
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = undefined;
    }
  }

  function handleClose() {
    stopAnimation();
    open = false;
  }
</script>

{#if open}
  <div class="backdrop" onclick={handleClose} role="presentation">
    <div
      class="panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="qr-modal-title"
      onclick={(e) => e.stopPropagation()}
    >
      <h2 id="qr-modal-title" class="title">Código QR del descriptor</h2>

      {#if loading}
        <p class="hint">Generando QR…</p>
      {:else if errorMessage}
        <p class="error" role="alert">{errorMessage}</p>
      {:else if frameImages.length > 0}
        <figure class="qr">
          <img src={frameImages[currentFrame]} alt="Código QR del descriptor" width="300" height="300" />
        </figure>
        {#if isAnimated}
          <p class="hint">
            QR animado BBQR — frame {currentFrame + 1} de {frameImages.length}.
            Apunta tu cámara a la pantalla; Sparrow rota frames automáticamente.
          </p>
        {:else}
          <p class="hint">QR estático — el descriptor cabe en un solo código.</p>
        {/if}
      {/if}

      <div class="actions">
        <button type="button" class="btn btn-secondary" onclick={handleClose}>Cerrar</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 9000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-md);
  }
  .panel {
    background: var(--color-surface-raised);
    border-radius: var(--radius-card);
    padding: var(--space-lg);
    max-width: 400px;
    width: 100%;
    box-shadow: var(--shadow-modal);
  }
  .title {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-heading);
    font-weight: var(--font-weight-bold);
    color: var(--color-text-primary);
  }
  .qr {
    margin: 0 0 var(--space-md) 0;
    display: flex;
    justify-content: center;
    background: white;
    padding: var(--space-md);
    border-radius: var(--radius-input);
    border: 1px solid var(--color-border);
  }
  .qr img {
    max-width: 100%;
    height: auto;
  }
  .hint {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    line-height: var(--line-height-label);
  }
  .error {
    margin: 0 0 var(--space-md) 0;
    font-size: var(--font-size-label);
    color: var(--color-warning-text);
    background: var(--color-warning-bg);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-input);
    border-left: 4px solid var(--color-warning-border);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-sm);
  }
  .btn {
    min-height: var(--touch-target);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    cursor: pointer;
    transition: background-color var(--transition-color), border-color var(--transition-color);
  }
  .btn-secondary {
    background: transparent;
    color: var(--color-text-primary);
    border: 1px solid var(--color-border);
  }
  .btn-secondary:hover { background: var(--color-surface-sunken); }
</style>
