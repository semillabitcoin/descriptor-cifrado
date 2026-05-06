<details class="threat">
  <summary>
    <span class="icon" aria-hidden="true">⚠</span>
    <span>Modelo de amenazas</span>
  </summary>
  <div class="content">
    <div class="callout">
      <strong>Ninguna ubicación debe contener simultáneamente el .bed y una xpub del multisig.</strong>
    </div>
    <div class="protege">
      <span class="prefix">✓ Protege:</span>
      Tu descriptor multisig contra un atacante que solo encuentre el archivo .bed.
    </div>
    <div class="caution">
      <span class="prefix">✗ No protege:</span>
      <ul>
        <li>Un atacante que comprometa StartOS durante el cifrado (el descriptor pasa por memoria del proceso).</li>
        <li>Un atacante que ya tenga una xpub de tu multisig.</li>
      </ul>
    </div>
    <a class="readme-link" href="#readme-threat-model">Ver modelo de amenazas completo →</a>
  </div>
</details>

<style>
  .threat {
    background: var(--color-surface-raised);
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-md) var(--space-lg);
  }
  summary {
    list-style: none;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--radius-button);
    font-size: var(--font-size-label);
    color: var(--color-text-secondary);
    min-height: var(--touch-target);
    transition: background-color var(--transition-color);
  }
  summary::-webkit-details-marker { display: none; }
  summary:hover { background: var(--color-surface-sunken); color: var(--color-text-primary); }
  .icon { font-size: 16px; }
  .content {
    margin-top: var(--space-md);
    display: flex; flex-direction: column; gap: var(--space-md);
    font-size: var(--font-size-body);
    line-height: var(--line-height-body);
  }
  .callout {
    background: var(--color-warning-bg);
    border-left: 4px solid var(--color-warning-border);
    color: var(--color-warning-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .protege {
    background: var(--color-success-bg);
    color: var(--color-success-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .caution {
    background: var(--color-caution-bg);
    color: var(--color-caution-text);
    padding: var(--space-md);
    border-radius: var(--radius-card);
  }
  .caution ul { margin: var(--space-sm) 0 0 0; padding-left: var(--space-lg); }
  .prefix { font-weight: var(--font-weight-bold); margin-right: var(--space-xs); }
  .readme-link {
    color: var(--color-accent);
    text-decoration: underline;
    font-size: var(--font-size-label);
  }
  .readme-link:hover { color: var(--color-accent-hover); }
</style>
