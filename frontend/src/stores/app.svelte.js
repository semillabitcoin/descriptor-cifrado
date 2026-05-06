// Estado global de la SPA (Svelte 5 runes pattern).
// Importar desde componentes: import { appState, ... } from '../stores/app.svelte.js';
//
// localStorage keys (UI-SPEC §LocalStorage Keys):
//   bed.theme           ∈ "light" | "dark" | "auto"   default "auto"
//   bed.historyEnabled  ∈ "true"  | "false"            default "false"
// NINGUNA otra clave se persiste — descriptor, xpub, resultados viven solo en memoria (D-16, D-17).

const VALID_TABS = ['cifrar', 'descifrar', 'historial'];
const VALID_THEMES = ['light', 'dark', 'auto'];

export const appState = $state({
  activeTab: 'cifrar',
  theme: 'auto',
  historyEnabled: false,
});

function applyThemeToDom(theme) {
  // tokens.css usa :root[data-theme="dark"] y :root:not([data-theme="light"]) @media dark.
  // Para 'auto' eliminamos el atributo (los media queries de tokens.css cubren auto).
  // Para 'light' y 'dark' lo seteamos explícitamente.
  const root = document.documentElement;
  if (theme === 'auto') {
    root.removeAttribute('data-theme');
  } else {
    root.setAttribute('data-theme', theme);
  }
}

export function initFromStorage() {
  try {
    const t = localStorage.getItem('bed.theme');
    if (VALID_THEMES.includes(t)) {
      appState.theme = t;
    }
    const h = localStorage.getItem('bed.historyEnabled');
    appState.historyEnabled = h === 'true';
  } catch {
    // localStorage no disponible (private mode, file://, etc.) — usar defaults.
  }
  applyThemeToDom(appState.theme);
}

export function setTheme(theme) {
  if (!VALID_THEMES.includes(theme)) return;
  appState.theme = theme;
  try {
    localStorage.setItem('bed.theme', theme);
  } catch {}
  applyThemeToDom(theme);
}

export function setHistoryEnabled(enabled) {
  appState.historyEnabled = !!enabled;
  try {
    localStorage.setItem('bed.historyEnabled', String(!!enabled));
  } catch {}
  // Si el usuario apaga el toggle estando en la tab Historial, volver a Cifrar (D-20).
  if (!enabled && appState.activeTab === 'historial') {
    appState.activeTab = 'cifrar';
  }
}

export function setActiveTab(tab) {
  if (!VALID_TABS.includes(tab)) return;
  // No permitir activar 'historial' si el toggle está OFF.
  if (tab === 'historial' && !appState.historyEnabled) return;
  appState.activeTab = tab;
}
