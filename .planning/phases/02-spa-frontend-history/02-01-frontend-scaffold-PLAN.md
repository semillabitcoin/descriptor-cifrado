---
phase: 02-spa-frontend-history
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - frontend/package.json
  - frontend/package-lock.json
  - frontend/.gitignore
  - frontend/vite.config.js
  - frontend/index.html
  - frontend/src/main.js
  - frontend/src/App.svelte
  - frontend/src/app.css
  - frontend/src/lib/tokens.css
  - frontend/src/assets/fonts/Inter.woff2
  - frontend/src/assets/fonts/JetBrainsMono.woff2
  - frontend/src/assets/fonts/LICENSE-Inter.txt
  - frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt
  - .gitignore
autonomous: true
requirements: [UI-01]
must_haves:
  truths:
    - "El comando `cd frontend && npm install` instala Svelte 5 + Vite y produce node_modules sin errores"
    - "El comando `cd frontend && npm run build` produce frontend/dist/index.html y assets hashed sin requests externos"
    - "Las fuentes Inter y JetBrains Mono viven en frontend/src/assets/fonts/ como woff2 variable"
    - "tokens.css declara la paleta exacta (light + dark) y la spacing scale del UI-SPEC"
    - "El bundle JS+CSS inicial (excluyendo fuentes) es <50 KB gzipped"
  artifacts:
    - path: "frontend/package.json"
      provides: "Manifest npm con svelte@5, vite@8, @sveltejs/vite-plugin-svelte@7"
      contains: '"svelte"'
    - path: "frontend/vite.config.js"
      provides: "Configuración Vite: plugin-svelte, proxy /api → 127.0.0.1:8080, assetsInlineLimit excluye woff2"
      contains: "127.0.0.1:8080"
    - path: "frontend/index.html"
      provides: "Entrada SPA con <div id=\"app\"> y script type=\"module\""
      contains: 'id="app"'
    - path: "frontend/src/lib/tokens.css"
      provides: "Custom properties: paleta light/dark, spacing scale, radii, shadows del UI-SPEC"
      contains: "--color-accent: #3B82F6"
    - path: "frontend/src/app.css"
      provides: "@font-face para Inter + JetBrains Mono self-hosted"
      contains: "@font-face"
    - path: "frontend/src/assets/fonts/Inter.woff2"
      provides: "Inter variable woff2 self-hosted (sin Google Fonts)"
    - path: "frontend/src/assets/fonts/JetBrainsMono.woff2"
      provides: "JetBrains Mono variable woff2 self-hosted"
  key_links:
    - from: "frontend/index.html"
      to: "frontend/src/main.js"
      via: '<script type="module" src="/src/main.js">'
      pattern: 'src="/src/main.js"'
    - from: "frontend/src/main.js"
      to: "frontend/src/App.svelte"
      via: "import App + mount(App, { target })"
      pattern: "mount\\(App"
    - from: "frontend/src/App.svelte"
      to: "frontend/src/app.css"
      via: "import './app.css'"
      pattern: "import.*app\\.css"
    - from: "frontend/src/app.css"
      to: "frontend/src/lib/tokens.css"
      via: "@import './lib/tokens.css'"
      pattern: "@import.*tokens"
---

<objective>
Establecer el esqueleto del frontend Svelte 5 + Vite 6 en `frontend/` top-level con tokens visuales (paleta light/dark + spacing scale del UI-SPEC), fuentes self-hosted (Inter + JetBrains Mono variable woff2), y configuración de build que produce `frontend/dist/` listo para `rust-embed`. Esta plan no toca el backend ni implementa componentes funcionales — solo el esqueleto, los tokens y la pipeline.

Purpose: Preparar la base sobre la que las planes 03–06 construirán la UI real. Sin esta base, ninguna plan downstream puede compilar Svelte ni renderizar tokens.
Output: `frontend/` con package.json, vite.config.js, index.html, main.js, App.svelte (placeholder), app.css con @font-face, tokens.css con todas las custom properties del UI-SPEC, y fuentes woff2 verificadas.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/02-spa-frontend-history/02-CONTEXT.md
@.planning/phases/02-spa-frontend-history/02-RESEARCH.md
@.planning/phases/02-spa-frontend-history/02-UI-SPEC.md
@CLAUDE.md
@.gitignore
</context>

<tasks>

<task type="auto" tdd="false">
  <name>Task 1: Inicializar proyecto frontend (package.json + vite.config.js + npm install)</name>
  <files>frontend/package.json, frontend/vite.config.js, frontend/.gitignore, frontend/index.html, frontend/src/main.js, frontend/src/App.svelte, .gitignore</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 2 — Vite + Svelte config; §Stack estándar — versiones)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-39, D-40, D-41, D-42)
    - CLAUDE.md (Tech stack lock-in: svelte 5, vite, rust-embed)
    - .gitignore (existing root .gitignore — añadir entradas de node_modules + dist; NO romper reglas existentes)
  </read_first>
  <action>
Crear el proyecto frontend en `frontend/` (top-level del repo, hermano de `crates/`). Pasos exactos:

1. Crear `frontend/package.json` con este contenido EXACTO:

```json
{
  "name": "bed-frontend",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "vite": "^8.0.0",
    "@sveltejs/vite-plugin-svelte": "^7.0.0"
  }
}
```

NO añadas `bbqr` aquí — se añade en plan 02-05 (Tab Descifrar) como dependencia normal con import dinámico.

2. Crear `frontend/vite.config.js` con este contenido EXACTO (proxy obligatorio para D-42, exclusión woff2 obligatoria para Trampa 2 del RESEARCH):

```javascript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': 'http://127.0.0.1:8080',
    },
  },
  build: {
    outDir: 'dist',
    target: 'es2022',
    sourcemap: false,
    cssCodeSplit: false,
    assetsInlineLimit: (filePath) => {
      if (/\.(woff2?|ttf|eot|otf)$/i.test(filePath)) return false;
      return 4096;
    },
    rollupOptions: {
      output: {
        assetFileNames: (assetInfo) => {
          const name = assetInfo.names?.[0] ?? assetInfo.name ?? '';
          if (/\.(woff2?|ttf|eot|otf)$/i.test(name)) {
            return 'assets/fonts/[name]-[hash][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
  },
});
```

3. Crear `frontend/index.html` con este contenido EXACTO (sin enlaces externos a fonts, scripts ni stylesheets — UI-01):

```html
<!doctype html>
<html lang="es">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>BED — Bitcoin Encrypted Backup</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.js"></script>
  </body>
</html>
```

4. Crear `frontend/src/main.js` (Svelte 5 usa `mount()`, NO `new App()` — RESEARCH §Patrón 2):

```javascript
import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

const app = mount(App, { target: document.getElementById('app') });
export default app;
```

5. Crear `frontend/src/App.svelte` placeholder mínimo (lo expande plan 02-03):

```svelte
<script>
  // Placeholder — Plan 02-03 lo reemplazará con header + tabs + threat model.
</script>

<main>
  <h1>BED</h1>
  <p>Frontend scaffold OK.</p>
</main>
```

6. Crear `frontend/.gitignore`:

```
node_modules/
dist/
.vite/
```

7. Añadir al `.gitignore` raíz (verificar primero qué hay; AÑADIR estas líneas si no existen, NO sobrescribir el archivo):

```
frontend/node_modules/
frontend/dist/
frontend/.vite/
```

8. Ejecutar la instalación: `cd /workspace/descriptor-cifrado/frontend && npm install`. Verificar que termina sin errores (exit code 0). Esto generará `frontend/package-lock.json` automáticamente — DEJARLO commiteado (es la lock file).

Notas:
- NO uses `npm init` interactivo — escribe el package.json directamente con el contenido literal de arriba.
- NO añadas `tailwindcss` ni ninguna otra dependencia — el UI-SPEC dice "Plain CSS custom properties + scoped Svelte `<style>`".
- NO añadas TypeScript — JavaScript plain (consistente con RESEARCH §Patrón 2).
- El servidor Vite dev se BIND a 127.0.0.1:5173 explícitamente (loopback only, consistente con SEC-02 del proyecto).
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend && npm install --no-audit --no-fund &amp;&amp; test -f package-lock.json &amp;&amp; test -d node_modules/svelte &amp;&amp; test -d node_modules/vite &amp;&amp; test -d node_modules/@sveltejs/vite-plugin-svelte</automated>
  </verify>
  <acceptance_criteria>
    - `test -f /workspace/descriptor-cifrado/frontend/package.json` retorna 0
    - `grep '"svelte": "\^5' /workspace/descriptor-cifrado/frontend/package.json` encuentra match
    - `grep '"vite": "\^8' /workspace/descriptor-cifrado/frontend/package.json` encuentra match
    - `grep '"@sveltejs/vite-plugin-svelte": "\^7' /workspace/descriptor-cifrado/frontend/package.json` encuentra match
    - `grep "127.0.0.1:8080" /workspace/descriptor-cifrado/frontend/vite.config.js` encuentra match (proxy D-42)
    - `grep "host: '127.0.0.1'" /workspace/descriptor-cifrado/frontend/vite.config.js` encuentra match (loopback bind)
    - `grep "woff2" /workspace/descriptor-cifrado/frontend/vite.config.js` encuentra match (exclusión inline)
    - `grep 'lang="es"' /workspace/descriptor-cifrado/frontend/index.html` encuentra match
    - `grep -E "https?://" /workspace/descriptor-cifrado/frontend/index.html` retorna sin matches (cero URLs externas)
    - `grep "mount(App" /workspace/descriptor-cifrado/frontend/src/main.js` encuentra match (Svelte 5 API)
    - `test -d /workspace/descriptor-cifrado/frontend/node_modules/svelte` retorna 0
    - `grep "frontend/node_modules" /workspace/descriptor-cifrado/.gitignore` encuentra match
  </acceptance_criteria>
  <done>npm install completa sin errores; svelte, vite y @sveltejs/vite-plugin-svelte instalados en versiones declaradas; vite.config.js declara proxy /api → 127.0.0.1:8080 y exclusión woff2 del inline; index.html sin URLs externas.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 2: Descargar e instalar fuentes self-hosted (Inter + JetBrains Mono variable woff2)</name>
  <files>frontend/src/assets/fonts/Inter.woff2, frontend/src/assets/fonts/JetBrainsMono.woff2, frontend/src/assets/fonts/LICENSE-Inter.txt, frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-02, D-43)
    - .planning/phases/02-spa-frontend-history/02-RESEARCH.md (Patrón 9 — Fuentes self-hosted)
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Typography)
  </read_first>
  <action>
Descargar las dos fuentes variable woff2 a `frontend/src/assets/fonts/`. NO usar Google Fonts CDN (rompe UI-01). NO usar `@fontsource` ni similar (añade dependencia npm innecesaria — las queremos como assets crudos para que rust-embed las sirva).

Pasos exactos:

1. Crear el directorio: `mkdir -p /workspace/descriptor-cifrado/frontend/src/assets/fonts/`

2. Descargar Inter variable (rsms.me oficial):
   - URL: `https://rsms.me/inter/font-files/InterVariable.woff2`
   - Comando: `curl -fSL -o /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2 https://rsms.me/inter/font-files/InterVariable.woff2`
   - Verificar tamaño >50 KB (la variable woff2 oficial pesa ~340 KB).

3. Descargar JetBrains Mono variable (release oficial GitHub):
   - URL primaria: `https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/variable/JetBrainsMono%5Bwght%5D.woff2`
   - Comando: `curl -fSL -o /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2 'https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/variable/JetBrainsMono%5Bwght%5D.woff2'`
   - Si falla, alternativa: descargar de release tag latest: `https://github.com/JetBrains/JetBrainsMono/releases/latest/download/JetBrainsMono.zip`, descomprimir, copiar `fonts/variable/JetBrainsMono[wght].woff2` → `JetBrainsMono.woff2`.
   - Verificar tamaño >50 KB.

4. Guardar las licencias (cumplimiento legal — ambas son OFL):
   - Inter LICENSE: `curl -fSL -o /workspace/descriptor-cifrado/frontend/src/assets/fonts/LICENSE-Inter.txt https://raw.githubusercontent.com/rsms/inter/master/LICENSE.txt`
   - JetBrains Mono LICENSE: `curl -fSL -o /workspace/descriptor-cifrado/frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt https://raw.githubusercontent.com/JetBrains/JetBrainsMono/master/OFL.txt`

5. Verificar que ambos woff2 NO son archivos HTML de error (a veces curl trae redirects con HTML):
   - Cabecera mágica woff2 son los bytes `wOF2` (0x77 0x4F 0x46 0x32) en el offset 0.
   - Comprobación: `head -c 4 /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2 | od -An -c` debe mostrar `w O F 2`.
   - Mismo check para JetBrainsMono.woff2.

NO inlinear las fuentes en CSS (Trampa 2 del RESEARCH — viola límite 50 KB del bundle).
NO descargar variantes estáticas múltiples (italic, bold, etc.) — la variable woff2 cubre el rango 100..900 con un solo archivo.
NO subir variantes static + variable simultáneamente — solo la variable woff2.
  </action>
  <verify>
    <automated>test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2 &amp;&amp; test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2 &amp;&amp; head -c 4 /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2 | grep -q wOF2 &amp;&amp; head -c 4 /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2 | grep -q wOF2</automated>
  </verify>
  <acceptance_criteria>
    - `test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2` retorna 0
    - `test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2` retorna 0
    - `wc -c /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2` muestra tamaño >50000 bytes
    - `wc -c /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2` muestra tamaño >50000 bytes
    - `head -c 4 /workspace/descriptor-cifrado/frontend/src/assets/fonts/Inter.woff2` empieza con bytes `wOF2` (firma woff2)
    - `head -c 4 /workspace/descriptor-cifrado/frontend/src/assets/fonts/JetBrainsMono.woff2` empieza con bytes `wOF2`
    - `test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/LICENSE-Inter.txt` retorna 0
    - `test -f /workspace/descriptor-cifrado/frontend/src/assets/fonts/LICENSE-JetBrainsMono.txt` retorna 0
  </acceptance_criteria>
  <done>Inter.woff2 y JetBrainsMono.woff2 descargados a frontend/src/assets/fonts/, con firma woff2 verificada y licencias OFL guardadas.</done>
</task>

<task type="auto" tdd="false">
  <name>Task 3: Tokens.css + app.css con paleta UI-SPEC, spacing, fonts y verificar build</name>
  <files>frontend/src/lib/tokens.css, frontend/src/app.css</files>
  <read_first>
    - .planning/phases/02-spa-frontend-history/02-UI-SPEC.md (§Spacing Scale, §Typography, §Color tokens, §Border Radius, §Shadows, §Transitions — TODAS las tablas)
    - .planning/phases/02-spa-frontend-history/02-CONTEXT.md (D-02, D-03 theme persist)
    - frontend/src/main.js (importa './app.css' — confirmar import path correcto)
    - frontend/src/App.svelte (placeholder actual)
  </read_first>
  <action>
Crear `frontend/src/lib/tokens.css` con TODAS las custom properties del UI-SPEC §Color, §Spacing Scale, §Border Radius, §Shadows, §Transitions. Crear `frontend/src/app.css` con @font-face self-hosted, reset mínimo y `@import "./lib/tokens.css"`. Estos archivos son la SINGLE SOURCE OF TRUTH visual — todos los componentes (planes 03–06) referencian estas variables.

1. `frontend/src/lib/tokens.css` — escribir LITERALMENTE con estos valores exactos extraídos del UI-SPEC §Color y §Spacing Scale:

```css
/* tokens.css — Single source of truth para color, spacing, radii, shadows, transitions.
   Valores derivados directamente de 02-UI-SPEC.md §Color, §Spacing Scale, §Border Radius,
   §Shadows, §Transitions. NO modificar sin actualizar UI-SPEC. */

:root {
  /* ===== Spacing scale (UI-SPEC §Spacing Scale, multiplos de 4 only) ===== */
  --space-xs: 4px;
  --space-sm: 8px;
  --space-sm-plus: 12px;
  --space-md: 16px;
  --space-lg: 24px;
  --space-xl: 32px;
  --space-2xl: 48px;
  --space-3xl: 64px;

  /* ===== Border radii ===== */
  --radius-button: 8px;
  --radius-input: 8px;
  --radius-card: 12px;
  --radius-modal: 12px;
  --radius-dropzone: 12px;
  --radius-toast: 8px;
  --radius-pill: 999px;

  /* ===== Touch target ===== */
  --touch-target: 44px;

  /* ===== Typography ===== */
  --font-sans: 'Inter', system-ui, -apple-system, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Code', 'Fira Code', 'Courier New', monospace;
  --font-size-label: 14px;
  --font-size-body: 16px;
  --font-size-heading: 20px;
  --font-size-display: 28px;
  --font-size-mono: 13px;
  --font-weight-regular: 400;
  --font-weight-bold: 600;
  --line-height-display: 1.1;
  --line-height-heading: 1.2;
  --line-height-label: 1.4;
  --line-height-body: 1.5;
  --line-height-mono: 1.6;

  /* ===== Shadows ===== */
  --shadow-header: 0 1px 3px rgba(0, 0, 0, 0.08);
  --shadow-card: 0 1px 2px rgba(0, 0, 0, 0.05);
  --shadow-modal: 0 8px 24px rgba(0, 0, 0, 0.18);
  --shadow-focus: 0 0 0 3px rgba(59, 130, 246, 0.25);

  /* ===== Transitions ===== */
  --transition-color: 150ms ease;
  --transition-focus: 100ms ease;
  --transition-disabled: 100ms ease;
  --transition-tab: 150ms ease;
  --transition-toggle: 150ms ease;
  --transition-toast-in: 200ms ease-out;
  --transition-toast-out: 150ms ease-in;
  --transition-details: 200ms ease;

  /* ===== Color tokens — LIGHT theme defaults ===== */
  --color-surface: #FAFAF9;
  --color-surface-raised: #FFFFFF;
  --color-surface-sunken: #F4F4F5;
  --color-border: #E4E4E7;
  --color-border-focus: #3B82F6;
  --color-text-primary: #18181B;
  --color-text-secondary: #71717A;
  --color-text-disabled: #A1A1AA;
  --color-accent: #3B82F6;
  --color-accent-hover: #2563EB;
  --color-accent-fg: #FFFFFF;
  --color-destructive: #EF4444;
  --color-destructive-hover: #DC2626;
  --color-destructive-fg: #FFFFFF;
  --color-warning-bg: #FEF9C3;
  --color-warning-border: #FDE047;
  --color-warning-text: #713F12;
  --color-success-bg: #F0FDF4;
  --color-success-text: #166534;
  --color-caution-bg: #FFFBEB;
  --color-caution-text: #92400E;
  --color-history-badge-bg: #DBEAFE;
  --color-history-badge-text: #1E40AF;
  --color-toast-bg: #18181B;
  --color-toast-text: #FAFAF9;
}

/* DARK theme — activated by [data-theme="dark"] on <html> (set by stores/app.svelte.js) */
:root[data-theme="dark"] {
  --color-surface: #18181B;
  --color-surface-raised: #27272A;
  --color-surface-sunken: #09090B;
  --color-border: #3F3F46;
  --color-border-focus: #60A5FA;
  --color-text-primary: #FAFAF9;
  --color-text-secondary: #A1A1AA;
  --color-text-disabled: #52525B;
  --color-accent: #60A5FA;
  --color-accent-hover: #93C5FD;
  --color-accent-fg: #09090B;
  --color-destructive: #F87171;
  --color-destructive-hover: #FCA5A5;
  --color-destructive-fg: #09090B;
  --color-warning-bg: #422006;
  --color-warning-border: #854D0E;
  --color-warning-text: #FDE68A;
  --color-success-bg: #052E16;
  --color-success-text: #86EFAC;
  --color-caution-bg: #1C1917;
  --color-caution-text: #FCD34D;
  --color-history-badge-bg: #1E3A5F;
  --color-history-badge-text: #93C5FD;
  --color-toast-bg: #FAFAF9;
  --color-toast-text: #18181B;
}

/* AUTO theme — follows OS prefers-color-scheme. Applied when data-theme="auto" or absent. */
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --color-surface: #18181B;
    --color-surface-raised: #27272A;
    --color-surface-sunken: #09090B;
    --color-border: #3F3F46;
    --color-border-focus: #60A5FA;
    --color-text-primary: #FAFAF9;
    --color-text-secondary: #A1A1AA;
    --color-text-disabled: #52525B;
    --color-accent: #60A5FA;
    --color-accent-hover: #93C5FD;
    --color-accent-fg: #09090B;
    --color-destructive: #F87171;
    --color-destructive-hover: #FCA5A5;
    --color-destructive-fg: #09090B;
    --color-warning-bg: #422006;
    --color-warning-border: #854D0E;
    --color-warning-text: #FDE68A;
    --color-success-bg: #052E16;
    --color-success-text: #86EFAC;
    --color-caution-bg: #1C1917;
    --color-caution-text: #FCD34D;
    --color-history-badge-bg: #1E3A5F;
    --color-history-badge-text: #93C5FD;
    --color-toast-bg: #FAFAF9;
    --color-toast-text: #18181B;
  }
}
```

2. `frontend/src/app.css` — @font-face self-hosted + reset mínimo + import tokens:

```css
@import './lib/tokens.css';

@font-face {
  font-family: 'Inter';
  src: url('./assets/fonts/Inter.woff2') format('woff2-variations'),
       url('./assets/fonts/Inter.woff2') format('woff2');
  font-weight: 100 900;
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'JetBrains Mono';
  src: url('./assets/fonts/JetBrainsMono.woff2') format('woff2-variations'),
       url('./assets/fonts/JetBrainsMono.woff2') format('woff2');
  font-weight: 100 800;
  font-style: normal;
  font-display: swap;
}

*, *::before, *::after {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  background: var(--color-surface);
  color: var(--color-text-primary);
  font-family: var(--font-sans);
  font-size: var(--font-size-body);
  line-height: var(--line-height-body);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

button, input, textarea, select {
  font-family: inherit;
}

:focus-visible {
  outline: 2px solid var(--color-border-focus);
  outline-offset: 2px;
}

/* Default visually-hidden helper for screen-reader-only text */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
```

3. Ejecutar `cd /workspace/descriptor-cifrado/frontend && npm run build` y verificar:
   - Salida `frontend/dist/index.html` existe
   - Salida `frontend/dist/assets/*.css` existe (con tokens)
   - `grep -E "https?://" frontend/dist/index.html` retorna sin matches (cero URLs externas)
   - `grep -rE "https?://(fonts\.googleapis|cdn\.|fonts\.gstatic)" frontend/dist/` retorna sin matches
   - El bundle JS+CSS gzipped (excluyendo woff2) cabe bajo 50 KB

NO uses CSS-in-JS, NO uses Tailwind, NO uses postcss plugins extra. Plain CSS only.
NO añadas reset complejo (normalize.css etc.) — el reset mínimo de arriba es suficiente.
  </action>
  <verify>
    <automated>cd /workspace/descriptor-cifrado/frontend &amp;&amp; npm run build 2>&amp;1 | tail -20 &amp;&amp; test -f dist/index.html &amp;&amp; ls dist/assets/*.css 1&gt;/dev/null 2&gt;&amp;1 &amp;&amp; ! grep -E "https?://" dist/index.html</automated>
  </verify>
  <acceptance_criteria>
    - `grep '\-\-space-md: 16px' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-space-lg: 24px' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-space-xl: 32px' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-color-accent: #3B82F6' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-color-destructive: #EF4444' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-color-warning-bg: #FEF9C3' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep '\-\-color-history-badge-bg: #DBEAFE' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep 'data-theme="dark"' /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep "prefers-color-scheme: dark" /workspace/descriptor-cifrado/frontend/src/lib/tokens.css` encuentra match
    - `grep "font-weight: 100 900" /workspace/descriptor-cifrado/frontend/src/app.css` encuentra match (Inter variable range)
    - `grep "font-weight: 100 800" /workspace/descriptor-cifrado/frontend/src/app.css` encuentra match (JetBrains Mono variable range)
    - `grep "@font-face" /workspace/descriptor-cifrado/frontend/src/app.css` encuentra al menos 2 matches
    - `grep "Inter.woff2" /workspace/descriptor-cifrado/frontend/src/app.css` encuentra match
    - `grep "JetBrainsMono.woff2" /workspace/descriptor-cifrado/frontend/src/app.css` encuentra match
    - `cd /workspace/descriptor-cifrado/frontend && npm run build` exit code 0
    - `test -f /workspace/descriptor-cifrado/frontend/dist/index.html` retorna 0
    - `grep -rE "fonts\.googleapis|fonts\.gstatic|cdn\.jsdelivr|unpkg\.com" /workspace/descriptor-cifrado/frontend/dist/` retorna sin matches
    - `grep -E "https?://" /workspace/descriptor-cifrado/frontend/dist/index.html` retorna sin matches
    - `find /workspace/descriptor-cifrado/frontend/dist/assets -name "*.woff2" | wc -l` retorna >=2 (las dos fuentes copiadas)
  </acceptance_criteria>
  <done>tokens.css declara la paleta exacta del UI-SPEC en light/dark/auto; app.css declara @font-face self-hosted; `npm run build` produce `dist/` sin URLs externas y con las fuentes copiadas como assets.</done>
</task>

</tasks>

<verification>
- `cd frontend && npm install` retorna exit code 0
- `cd frontend && npm run build` retorna exit code 0 y produce `frontend/dist/index.html`
- `grep -rE "https?://" frontend/dist/index.html` no encuentra URLs externas
- `find frontend/dist/assets -name "*.woff2"` lista las dos fuentes embebidas como assets locales
- Los tokens CSS y la paleta exacta del UI-SPEC viven en `frontend/src/lib/tokens.css`
</verification>

<success_criteria>
- frontend/ scaffold listo: package.json, vite.config.js (proxy /api → 127.0.0.1:8080), index.html, main.js, App.svelte placeholder
- Fuentes Inter y JetBrains Mono variable woff2 self-hosted (sin Google Fonts CDN)
- tokens.css con paleta exacta light/dark/auto del UI-SPEC, spacing scale, radii, shadows, transitions
- `npm run build` produce dist/ sin URLs externas (UI-01 verificado)
</success_criteria>

<output>
After completion, create `.planning/phases/02-spa-frontend-history/02-01-SUMMARY.md` describing:
- Tooling versions resueltas por npm (svelte X.Y.Z, vite X.Y.Z, plugin-svelte X.Y.Z)
- Tamaños finales: Inter.woff2 (KB), JetBrainsMono.woff2 (KB), bundle dist/assets/*.css gzipped (KB)
- Confirmación: 0 URLs externas en dist/
- Path absoluto a tokens.css y app.css para referenciar desde planes 03–06
</output>
