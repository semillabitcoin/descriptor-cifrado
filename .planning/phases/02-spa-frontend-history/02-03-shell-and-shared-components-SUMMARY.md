---
phase: 02-spa-frontend-history
plan: 03
subsystem: frontend
tags: [frontend, svelte5, runes, shell, components, aria, tokens, ui-02, ui-03, hist-01]

requires:
  - phase: 02-spa-frontend-history
    plan: 01
    provides: "frontend/src/lib/tokens.css custom properties; frontend/src/app.css con @font-face; Vite scaffold"
provides:
  - "frontend/src/stores/app.svelte.js — \$state global con activeTab/theme/historyEnabled + initFromStorage/setTheme/setHistoryEnabled/setActiveTab"
  - "frontend/src/lib/api.js — ApiError class + postJson/postMultipart/getJson/deleteJson"
  - "frontend/src/lib/clipboard.js — copyToClipboard con fallback execCommand"
  - "frontend/src/components/{Spinner,Toast,InlineError,Modal,ThreatModel,ThemeToggle,HistoryToggle,HistoryBadge,Header,TabBar}.svelte"
  - "frontend/src/App.svelte — root SPA con Header + ThreatModel + TabBar + 3 tabpanels"
affects: [02-04-tab-cifrar, 02-05-tab-descifrar, 02-06-tab-historial-and-rust-embed]

tech-stack:
  added: []
  patterns:
    - "Svelte 5 runes \$state global con archivos .svelte.js (no writable() de Svelte 4)"
    - "ARIA tablist/tab/tabpanel con roving tabindex + keyboard nav (ArrowLeft/Right/Home/End)"
    - "Modal focus trap manual con default focus en Cancel + Escape para cerrar (D-36)"
    - "Toast aria-live=polite + auto-dismiss 3s + slide-in animation"
    - "Inline error con role=alert"
    - "Clipboard API con fallback execCommand para contextos no-secure (Tor onion HTTP)"
    - "ApiError class con .code/.message/.status para envelope error de backend"
    - "<details>/<summary> nativos para threat model collapsible (D-30)"
    - "data-theme attribute en <html> para light/dark; ausente para auto (media query)"

key-files:
  created:
    - "frontend/src/stores/app.svelte.js"
    - "frontend/src/lib/api.js"
    - "frontend/src/lib/clipboard.js"
    - "frontend/src/components/Spinner.svelte"
    - "frontend/src/components/Toast.svelte"
    - "frontend/src/components/InlineError.svelte"
    - "frontend/src/components/Modal.svelte"
    - "frontend/src/components/ThreatModel.svelte"
    - "frontend/src/components/ThemeToggle.svelte"
    - "frontend/src/components/HistoryToggle.svelte"
    - "frontend/src/components/HistoryBadge.svelte"
    - "frontend/src/components/Header.svelte"
    - "frontend/src/components/TabBar.svelte"
  modified:
    - "frontend/src/App.svelte"

decisions:
  - "Svelte a11y warnings 'no_noninteractive_element_to_interactive_role' suprimidos con svelte-ignore en <nav role=tablist> y <section role=tabpanel>: pattern WAI-ARIA estándar, falso positivo del linter Svelte"
  - "ThreatModel renderizado como banner directo bajo Header (no dentro del header) para que el <details> ocupe full-width al expandir; el trigger <summary> hace de botón accesible"
  - "Tab Historial NO se renderiza en DOM cuando historyEnabled=false (no solo hidden) — D-20: must_haves enforce 'NO solo hidden'"
  - "ApiError class extiende Error para mantener stack trace + .code/.status custom; networkError unifica TypeError de fetch a NETWORK_ERROR (mensaje localizado)"
  - "Modal usa queueMicrotask para focus en cancelBtn (asegura mount completo antes de focus)"

requirements-completed: [UI-02, UI-03, HIST-01]

metrics:
  duration_minutes: 4
  tasks_completed: 3
  commits: 3
  files_created: 13
  files_modified: 1
  completed: 2026-05-06
---

# Phase 02 Plan 03: Shell and Shared Components Summary

Shell SPA Svelte 5 + 8 componentes compartidos + store global $state con persistencia localStorage para `bed.theme` y `bed.historyEnabled` solamente. Bundle JS+CSS gzipped = 18,439 bytes (well under 50 KB budget).

## Outcome

`cd frontend && npm run build` produce dist/:

```
dist/index.html                                   0.41 KB │ gzip:  0.28 KB
dist/assets/style-B-H8Sio6.css                    9.32 KB │ gzip:  2.38 KB
dist/assets/index-CQKP89bg.js                    41.92 KB │ gzip: 16.27 KB
dist/assets/fonts/Inter-DiVDrmQJ.woff2          352.24 KB
dist/assets/fonts/JetBrainsMono-BeqGHA24.woff2  113.67 KB
```

**Bundle JS+CSS gzipped (excluyendo fonts) = 18,439 bytes / 18.0 KB.** UI-SPEC §Build Constraints exige <50 KB → **36% del budget consumido**, 32 KB libres para los planes 04/05/06.

## Exports para planes downstream (04/05/06)

### `frontend/src/stores/app.svelte.js`

```javascript
import {
  appState,           // $state { activeTab, theme, historyEnabled }
  initFromStorage,    // () => void — llamar onMount en App.svelte (ya hecho)
  setTheme,           // (theme: 'light'|'dark'|'auto') => void
  setHistoryEnabled,  // (enabled: boolean) => void
  setActiveTab,       // (tab: 'cifrar'|'descifrar'|'historial') => void
} from '../stores/app.svelte.js';
```

`appState.activeTab` ∈ `'cifrar' | 'descifrar' | 'historial'`. Los planes 04/05/06 leen para mostrar/ocultar contenido.

### `frontend/src/lib/api.js`

```javascript
import { ApiError, postJson, postMultipart, getJson, deleteJson } from '../lib/api.js';
```

- `postJson(url, body)` — POST application/json, retorna body parseado (JSON) o lanza `ApiError`.
- `postMultipart(url, formData)` — POST multipart, lo mismo.
- `getJson(url)` — GET, retorna JSON o `ApiError`.
- `deleteJson(url)` — DELETE, retorna `null` (204) o lanza `ApiError`.
- `ApiError`: `{ name: 'ApiError', status: number, code: string, message: string }`.

`code` es el campo `error.code` del envelope backend (UPPER_SNAKE_CASE). Los planes downstream pueden hacer `try { ... } catch (e) { if (e instanceof ApiError && e.code === 'INVALID_DESCRIPTOR') { ... } }`.

### `frontend/src/lib/clipboard.js`

```javascript
import { copyToClipboard } from '../lib/clipboard.js';
const ok = await copyToClipboard(text); // boolean
```

Usa `navigator.clipboard.writeText` cuando `isSecureContext`, fallback a `execCommand('copy')` en `<textarea>` efímero para Tor onion HTTP.

## Estructura de paneles en App.svelte (mounting points para planes 04/05/06)

```svelte
<!-- App.svelte (ya existe) -->
<Header />
<ThreatModel />
<main>
  <TabBar />
  <section id="panel-cifrar" hidden={appState.activeTab !== 'cifrar'}>
    <!-- Plan 02-04 monta aquí: <TabCifrar /> -->
    <p class="placeholder">Tab Cifrar — pendiente de plan 02-04.</p>
  </section>
  <section id="panel-descifrar" hidden={appState.activeTab !== 'descifrar'}>
    <!-- Plan 02-05 monta aquí: <TabDescifrar /> -->
    <p class="placeholder">Tab Descifrar — pendiente de plan 02-05.</p>
  </section>
  {#if appState.historyEnabled}
    <section id="panel-historial" hidden={appState.activeTab !== 'historial'}>
      <!-- Plan 02-06 monta aquí: <TabHistorial /> -->
      <p class="placeholder">Tab Historial — pendiente de plan 02-06.</p>
    </section>
  {/if}
</main>
```

Los planes downstream solo necesitan:
1. Crear `TabCifrar.svelte` / `TabDescifrar.svelte` / `TabHistorial.svelte` en `frontend/src/components/`.
2. Importarlos en App.svelte.
3. Reemplazar el `<p class="placeholder">...</p>` correspondiente con `<TabCifrar />` (etc.).

Los componentes comprartidos `Modal`, `Toast`, `InlineError`, `Spinner` ya están listos para consumir.

## Componentes compartidos (contracts)

| Componente | Props clave | ARIA |
|------------|-------------|------|
| Spinner | `size=16`, `color=var(--color-accent-fg)` | `role=status`, `aria-label="Cargando"` |
| Toast | `message`, `visible=$bindable(false)`, `durationMs=3000` | `role=status`, `aria-live="polite"` |
| InlineError | `message`, `visible=$bindable(false)` | `role="alert"` |
| Modal | `open=$bindable(false)`, `title`, `children`, `onConfirm`, `onCancel`, `confirmLabel`, `cancelLabel`, `confirmVariant='primary'\|'destructive'`, `confirmLoading=false` | `role="dialog"`, `aria-modal="true"`, `aria-labelledby="modal-title"`, focus trap, Escape closes |
| ThreatModel | (no props) | `<details>/<summary>` nativos |
| ThemeToggle | (no props, lee/escribe appState) | `aria-label` dinámico según tema |
| HistoryToggle | (no props, lee/escribe appState) | `role="switch"`, `aria-checked` |
| HistoryBadge | (no props, condicional según appState.historyEnabled) | — |
| Header | (no props) | semantic `<header>` |
| TabBar | (no props) | `role="tablist"`, `role="tab"`, `aria-selected`, `aria-controls`, keyboard nav |

## Commits

| Task | Hash    | Message                                                                |
| ---- | ------- | ---------------------------------------------------------------------- |
| 1    | 4b0e8d9 | feat(02-03): add svelte 5 store + lib/api + lib/clipboard              |
| 2    | a791041 | feat(02-03): add 8 shared UI components                                |
| 3    | de3b1f4 | feat(02-03): wire App shell with Header + ThreatModel + TabBar         |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 — Blocking] Svelte a11y false-positive warnings on standard ARIA tab pattern**
- **Found during:** Task 3 (npm run build)
- **Issue:** Svelte's a11y linter emits `a11y_no_noninteractive_element_to_interactive_role` warnings for `<nav role="tablist">` y `<section role="tabpanel">`. El plan exige estos roles ARIA explícitamente (D-07 + UI-SPEC §Tabs); estos son patterns WAI-ARIA estándar y correctos. Las warnings ensucian el output de build aunque no son errors.
- **Fix:** Añadí `<!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->` antes de cada `<nav>` y `<section>` afectado en `TabBar.svelte` y `App.svelte`. La supresión es local (no global) y el comentario documenta la intención.
- **Files modified:** `frontend/src/components/TabBar.svelte`, `frontend/src/App.svelte`
- **Verification:** `npm run build` ahora corre limpio sin warnings.
- **Commit:** `de3b1f4` (Task 3 commit)

## Verification Summary

```bash
$ cd frontend && npm run build
# Build verde, 0 warnings, 0 errors

$ # Bundle size
Bundle JS+CSS gzipped: 18,439 bytes (UI-SPEC budget <51,200)

$ # ARIA roles aplicados (TabBar, App, componentes)
role="tablist" role="tab" role="tabpanel" role="dialog" role="alert"
role="status" role="switch" aria-modal aria-checked aria-selected aria-controls
aria-live="polite" aria-labelledby="modal-title"

$ # Copy literal del UI-SPEC §Threat Model Copy
"Ninguna ubicación debe contener simultáneamente el .bed y una xpub del multisig."
"✓ Protege:" "✗ No protege:" "Ver modelo de amenazas completo →"
"Modo histórico activo"
```

## Self-Check: PASSED

- frontend/src/stores/app.svelte.js: FOUND
- frontend/src/lib/api.js: FOUND
- frontend/src/lib/clipboard.js: FOUND
- frontend/src/components/Spinner.svelte: FOUND
- frontend/src/components/Toast.svelte: FOUND
- frontend/src/components/InlineError.svelte: FOUND
- frontend/src/components/Modal.svelte: FOUND
- frontend/src/components/ThreatModel.svelte: FOUND
- frontend/src/components/ThemeToggle.svelte: FOUND
- frontend/src/components/HistoryToggle.svelte: FOUND
- frontend/src/components/HistoryBadge.svelte: FOUND
- frontend/src/components/Header.svelte: FOUND
- frontend/src/components/TabBar.svelte: FOUND
- frontend/src/App.svelte: MODIFIED (placeholder replaced)
- commit 4b0e8d9: FOUND
- commit a791041: FOUND
- commit de3b1f4: FOUND
- frontend/dist/index.html: FOUND (post-build)
- Bundle JS+CSS gzipped: 18,439 bytes < 51,200 (50 KB budget) PASS
