# Phase 2: SPA Frontend + History — Research

**Investigado:** 2026-05-06
**Dominio:** Svelte 5 SPA + Vite 6 + rust-embed 8 + axum history endpoints
**Confianza general:** HIGH (stack bloqueado; la mayoría de decisiones ya tomadas en CONTEXT.md)

---

<user_constraints>
## Restricciones del usuario (desde CONTEXT.md)

### Decisiones bloqueadas (D-01 a D-43)

- **D-01:** Mobile-first responsive (360/768/1024+), touch-friendly ≥44×44 px, look profesional minimal cálido.
- **D-02:** Tipografía self-hosted: Inter (body/headings) + JetBrains Mono (descriptors, xpubs, IDs). woff2 variable. NO Google Fonts CDN.
- **D-03:** Tema light/dark/auto siguiendo `prefers-color-scheme`. Toggle manual. Persiste en `localStorage` clave `bed.theme`.
- **D-05:** Header global + 2-3 tabs. Tab "Historial" condicional (solo si toggle history ON).
- **D-06:** Routing state-based interno con stores Svelte. Sin hash routing.
- **D-07:** Tabs con roles ARIA estándar (`role="tablist/tab/tabpanel"`, `aria-selected`, `aria-controls`).
- **D-08:** Tab Cifrar: textarea descriptor + botón "Cifrar". Validación inline con mensaje del backend.
- **D-09:** Tras "Cifrar" exitoso, tres outputs simultáneos: `.bed` download, armored con "Copiar", QR PNG inline + "Descargar PNG".
- **D-10:** Sin pre-selección de formato. Los tres se muestran siempre.
- **D-11:** Si QrTooLarge (422), mostrar mensaje del backend + sugerencia. Sin BBQR fallback en Cifrar v1.
- **D-12:** Si history toggle ON al cifrar: segunda llamada `POST /api/history`. Si falla, warning sin invalidar resultado.
- **D-13:** Tab Descifrar: drop-zone + textarea armored + file picker. Sin stepper.
- **D-14:** "Descifrar" disabled hasta `.bed` no vacío + xpub válida (regex cliente).
- **D-15:** Tras descifrar: descriptor en `<pre>` + "Copiar", "Descargar .txt", "Mostrar QR" (BBQR si excede capacidad).
- **D-16:** Descriptor recuperado solo en memoria navegador. Desaparece al cambiar tab o recargar.
- **D-17:** xpub nunca persiste. Se limpia automáticamente tras descifrado.
- **D-18:** Toggle history OFF por defecto (`bed.historyEnabled` = `"false"`). Badge "Modo histórico activo" cuando ON.
- **D-19:** Backend SIN estado global del toggle. Toggle 100% client-side. API stateless.
- **D-20:** Tab Historial oculta en DOM si toggle OFF.
- **D-21:** Lista de entradas con timestamp relativo + tooltip ISO, short-id, botones "Ver" / "Borrar".
- **D-22:** "Ver" regenera los tres formatos on-demand desde el `.bed` persistido (server-side).
- **D-23:** "Borrar" → modal de confirmación → `DELETE /api/history/:id` → toast.
- **D-24:** Empty state si lista vacía (texto + icono; sin ilustración pesada).
- **D-25–D-29:** Contratos de los 4 endpoints nuevos (ver sección API Contract más abajo).
- **D-30–D-32:** Threat model panel colapsable vía `<details>`, contenido en orden definido.
- **D-33:** Loading: spinner inline dentro del botón. Sin overlay full-page.
- **D-34:** Copy: toast 3s + label change a "Copiado ✓" por 1500 ms.
- **D-35:** Errores API: alerta inline arriba del form, cerrable.
- **D-36:** Confirmación destructiva: modal con foco en "Cancelar". Requiere segundo click en "Borrar".
- **D-37–D-38:** WCAG AA: `<label for>`, HTML semántico, contraste, focus visible, `aria-live`, no-color-only.
- **D-39:** Frontend en `frontend/` top-level del repo.
- **D-40:** `#[derive(RustEmbed)] #[folder = "../../frontend/dist/"]` en `crates/server`.
- **D-41:** Vite emite hashed filenames. `index.html` referencia los hashes.
- **D-42:** Dev mode: Vite dev server en 5173 con proxy `/api` → `127.0.0.1:8080`.
- **D-43:** Fonts en `frontend/src/assets/fonts/` como woff2 variable. Cero requests externos.

### A discreción de Claude
- Paleta exacta de colores (ya definida en UI-SPEC.md).
- Spacing scale (ya definida en UI-SPEC.md).
- Estructura interna de stores Svelte (un store global vs múltiples).
- Componentización interna (granularidad de componentes).
- Patrón de manejo de errores fetch (try/catch + error store).
- Estilo visual del modal, toast, alerts (ya definido en UI-SPEC.md).

### Ideas diferidas (FUERA DE ALCANCE Phase 2)
- Persistencia cross-restart del toggle history en backend (PERS-01).
- BBQR fallback en Cifrar (cuando `.bed` excede 2900 B QR).
- UX2-01 drag-and-drop avanzado, UX2-02 test decrypt, UX2-03 checksum visual, UX2-04 errores específicos.
- Camera scanner QR (requiere secure context HTTPS).
- Exportar history como ZIP.
- Multi-language i18n.
- Themes custom adicionales.
- Dockerfile, GHCR, s9pk (Phase 3 + Phase 4).
</user_constraints>

---

<phase_requirements>
## Requisitos de Phase 2

| ID | Descripción | Soporte de investigación |
|----|-------------|--------------------------|
| UI-01 | SPA Svelte 5 + Vite 6 servida desde el binario vía rust-embed, sin CDN externo, sin telemetría, sin fonts remotas | Ver §rust-embed + axum + §Pipeline Vite |
| UI-02 | La UI presenta dos pestañas: "Cifrar" y "Descifrar" | Ver §Svelte 5 Runes + §Tabs ARIA |
| UI-03 | La UI muestra modelo de amenazas resumido visible | Ver §Componentes: `<details>` HTML semántico |
| HIST-01 | Toggle en la UI activa modo "guardar historial"; default es ephemeral | Ver §localStorage + §Toggle Svelte |
| HIST-02 | Con toggle activo, los `.bed` resultantes se persisten en `/data/encrypted/<timestamp>-<short-id>.bed` | Ver §History Endpoints Rust |
| HIST-03 | El descriptor en claro NUNCA se persiste en disco (test CI hace grep) | Ver §Seguridad: no-leak test extendido |
| HIST-04 | Endpoint `GET /api/history` lista entradas vía directory scan | Ver §Directory Scan vs redb |
| HIST-05 | Endpoint `DELETE /api/history/:id` borra una entrada | Ver §History Endpoints Rust |
| HIST-06 | La UI lista y permite borrar entradas del historial | Ver §Svelte 5 Historia Tab |
</phase_requirements>

---

## Resumen ejecutivo

Phase 2 añade dos capas ortogonales: (1) un frontend Svelte 5 compilado por Vite 6 y embebido en el binario Rust mediante `rust-embed 8`, y (2) cuatro endpoints Rust nuevos para el historial opt-in de `.bed` cifrados. El stack ya está bloqueado en CONTEXT.md y CLAUDE.md; la investigación confirma que todas las decisiones son técnicamente sólidas y proporciona los patrones exactos de implementación.

La mayor ambigüedad resuelta aquí es: **no se usará `redb` en Phase 2**. El historial solo necesita listar y borrar entradas — un directory scan de `/data/encrypted/` es suficiente y elimina una dependencia entera. `redb` es la alternativa para v1.x cuando se necesiten consultas más ricas, y ya está en el stack como opción. CONTEXT.md (D-27) ya documenta explícitamente que `GET /api/history` se implementa como directory scan.

El segundo tema a resolver es el BBQR en el lado cliente (Descifrar tab, D-15). El paquete npm `bbqr@1.2.0` de Coinkite es ESM nativo, funciona en browser, tiene ~123 KB comprimido (tarball completo; el tree-shaking de Vite reducirá el impacto real). Dado el límite de 50 KB para la SPA completa, se recomienda evaluar si se implementa BBQR inline o se activa lazy en demanda.

**Recomendación primaria:** Estructura el trabajo en dos streams paralelos: (A) pipeline frontend Vite → rust-embed → axum, y (B) handlers Rust para los 4 endpoints de history. El stream A es el crítico para UI-01; el stream B es el crítico para HIST-02/04/05.

---

## Stack estándar

### Frontend (build-time, no en imagen runtime)

| Biblioteca | Versión verificada | Propósito | Por qué es estándar |
|------------|--------------------|-----------|---------------------|
| `svelte` | 5.55.5 (npm) | Compilador SPA reactivo | Sin runtime framework — produce JS/CSS puro; runes para reactividad sin boilerplate |
| `vite` | 8.0.10 (npm) | Bundler + dev server | Estándar para Svelte 5 plain (no SvelteKit); emite `dist/` que rust-embed ingiere |
| `@sveltejs/vite-plugin-svelte` | 7.1.1 (npm) | Plugin Vite para compilar `.svelte` | Co-mantenido con Svelte; soporta Svelte 5 runes completo |
| `bbqr` | 1.2.0 (npm) | BBQR animated QR encode/decode (solo Descifrar tab) | ESM nativo browser, Coinkite (spec author), Public Domain |

**Nota versiones:** Confirmadas con `npm view [pkg] version` el 2026-05-06. Svelte usa el campo `svelte@5.55.5`; Vite usa el campo `vite@8.0.10`.

### Backend — dependencias nuevas en `crates/server/Cargo.toml`

| Crate | Versión verificada | Propósito | Notas |
|-------|--------------------|-----------|-------|
| `rust-embed` | 8.x (8.8.0+ en crates.io) | Embeder assets frontend en el binario | Feature `axum-ex` para compatibilidad con axum 0.8 |
| `mime_guess` | 2.x (transitiva de rust-embed axum-ex) | MIME types para assets embebidos | Ya en grafo de dependencias de `rust-embed` con `axum-ex` |
| `uuid` | 1.x | Short-id de 8 caracteres para entradas historial | Feature `v4` |
| `tokio` fs features | ya en workspace | Leer directorio `/data/encrypted/` | Feature `io-util` ya en workspace; añadir `fs` si se usa `tokio::fs` |

**No se añade `redb`** en Phase 2. Directory scan es suficiente para list+delete (confirmado en D-27).

### Instalación frontend

```bash
cd frontend
npm install svelte@5 vite@8 @sveltejs/vite-plugin-svelte@7
# Para BBQR (Descifrar tab, on-demand):
npm install bbqr@1.2.0
```

**Instalación Cargo** (añadir a `crates/server/Cargo.toml`):
```toml
rust-embed = { version = "8", features = ["axum-ex"] }
uuid = { version = "1", features = ["v4"] }
# Si se usa tokio::fs::read_dir:
tokio = { workspace = true, features = ["rt-multi-thread", "macros", "io-util", "net", "signal", "fs"] }
```

### Añadir al `[workspace.dependencies]` en `Cargo.toml` raíz:
```toml
rust-embed = { version = "8", features = ["axum-ex"] }
uuid = { version = "1", features = ["v4"] }
```

---

## Patrones de arquitectura

### Estructura de directorios propuesta

```
descriptor-cifrado/
├── Cargo.toml                      # workspace — añadir rust-embed, uuid
├── crates/
│   ├── core/                       # Phase 1, sin cambios
│   └── server/
│       ├── Cargo.toml              # añadir rust-embed, uuid
│       └── src/
│           ├── assets.rs           # NUEVO: RustEmbed derive + axum service
│           ├── lib.rs              # ampliar router con 4 rutas history + assets
│           ├── main.rs             # sin cambios
│           ├── error.rs            # añadir HistoryNotFound, HistoryWriteFailed, HistoryInvalidId
│           ├── state.rs            # añadir BED_DATA_DIR env var accessor
│           └── routes/
│               ├── encrypt.rs      # sin cambios
│               ├── decrypt.rs      # sin cambios
│               ├── history_post.rs # NUEVO: POST /api/history
│               ├── history_get.rs  # NUEVO: GET /api/history
│               ├── history_get_id.rs # NUEVO: GET /api/history/:id
│               └── history_delete.rs # NUEVO: DELETE /api/history/:id
└── frontend/                       # NUEVO
    ├── package.json
    ├── vite.config.js
    ├── index.html
    └── src/
        ├── main.js                 # entry: mounts App.svelte
        ├── App.svelte              # root: header + tabs + threat model
        ├── stores/
        │   └── app.svelte.js       # estado global reactivo (theme, historyEnabled, tab activa)
        ├── components/
        │   ├── Header.svelte
        │   ├── TabBar.svelte
        │   ├── TabCifrar.svelte
        │   ├── TabDescifrar.svelte
        │   ├── TabHistorial.svelte
        │   ├── ThreatModel.svelte
        │   ├── Modal.svelte
        │   ├── Toast.svelte
        │   └── InlineError.svelte
        └── assets/
            └── fonts/
                ├── Inter.woff2
                └── JetBrainsMono.woff2
```

---

### Patrón 1: rust-embed + axum — serving SPA con fallback

**Qué es:** `rust-embed 8` con feature `axum-ex` expone el pattern de `EmbedableFile` + `IntoResponse`. El ejemplo oficial (`cargo run --example axum --features axum-ex`) muestra tres rutas: `/` y `/index.html` → handler que sirve `index.html` embebido; `/{*file}` → handler que sirve cualquier asset embebido; fallback → 404. Para una SPA sin client-side routing (D-06 usa state-based, no URL routing), este patrón es suficiente — solo se necesita que `/` sirva `index.html` y que los assets hashed de Vite se sirvan correctamente.

**Cuándo:** Al montar el router en `lib.rs` (Wave "Assets").

**Código base verificado (fuente: docs.rs/crate/rust-embed/latest/source/examples/axum.rs):**

```rust
// crates/server/src/assets.rs
use axum::{
    body::Body,
    http::{header, Response, StatusCode},
    response::IntoResponse,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../frontend/dist/"]
pub struct FrontendAssets;

pub struct StaticFile<T>(pub T);

impl<T: Into<String>> IntoResponse for StaticFile<T> {
    fn into_response(self) -> Response<Body> {
        let path: String = self.0.into();
        match FrontendAssets::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(&path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(Body::from(content.data))
                    .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
            }
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }
}

// En lib.rs — handler para /
pub async fn index_handler() -> impl IntoResponse {
    StaticFile("index.html")
}

// En lib.rs — handler para /{*file}
pub async fn static_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    StaticFile(path)
}
```

```rust
// En lib.rs, router():
Router::new()
    .route("/", get(index_handler))
    .route("/index.html", get(index_handler))
    .route("/{*file}", get(static_handler))  // assets hashed de Vite
    .route("/api/encrypt", post(routes::encrypt::post_encrypt))
    .route("/api/decrypt", post(routes::decrypt::post_decrypt))
    .route("/api/history", post(routes::history_post::post_history))
    .route("/api/history", get(routes::history_get::get_history))
    .route("/api/history/:id", get(routes::history_get_id::get_history_id))
    .route("/api/history/:id", delete(routes::history_delete::delete_history))
    .layer(DefaultBodyLimit::max(512 * 1024))
```

**Nota importante:** Las rutas `/api/*` se declaran ANTES o DESPUÉS del wildcard `/{*file}`. En axum, los matchers más específicos tienen precedencia — `/api/encrypt` es más específico que `/{*file}`, por lo que el orden en el builder no afecta el routing. Pero por claridad y mantenibilidad, declarar API routes primero.

**CRÍTICO: Ruta del folder en RustEmbed.** Con la estructura de workspace donde `crates/server/` es el crate y `frontend/dist/` es relativo a la raíz del workspace, la ruta correcta es `#[folder = "../../frontend/dist/"]` (dos niveles arriba desde `crates/server/`). Esta ruta se resuelve en tiempo de compilación relativa al `Cargo.toml` del crate. Confirmar con `cargo build` desde la raíz del workspace.

**Dev mode (D-42):** En dev, `rust-embed` con feature `debug-embed` lee del filesystem en lugar de embeder. Sin ese feature, el binario de dev intentará embeber desde `frontend/dist/` en compile time. La solución estándar es ejecutar `vite build --watch` en una terminal y `cargo run` en otra, sin el flag `debug-embed`. O bien usar `ServeDir` de tower-http apuntando a `frontend/dist/` condicionalmente en dev:

```rust
// Alternativa dev: condicional en cfg(debug_assertions)
#[cfg(debug_assertions)]
fn assets_service() -> /* tower service */ { /* ServeDir de frontend/dist/ */ }
#[cfg(not(debug_assertions))]
fn assets_service() -> /* tower service */ { /* StaticFile embed */ }
```

La opción más simple en v1 (dados los constraints): compilar siempre desde embed, usar `vite build --watch` para regenerar `dist/` en dev, y `cargo watch -x run` para recompilar el binario automáticamente.

**Confianza:** HIGH — código del ejemplo oficial de rust-embed confirmado en docs.rs.

---

### Patrón 2: Vite 6 + Svelte 5 — configuración mínima

**Qué es:** Proyecto plain Svelte 5 (sin SvelteKit) con Vite 6. La diferencia clave con SvelteKit: no hay `adapter-static`, no hay `routes/`, no hay SSR. Solo un `index.html`, un `src/main.js` y componentes `.svelte`.

**`frontend/package.json`:**
```json
{
  "name": "bed-frontend",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview"
  },
  "devDependencies": {
    "svelte": "^5.0.0",
    "vite": "^8.0.0",
    "@sveltejs/vite-plugin-svelte": "^7.0.0"
  },
  "dependencies": {
    "bbqr": "^1.2.0"
  }
}
```

**`frontend/vite.config.js`:**
```javascript
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8080',
    },
  },
  build: {
    outDir: 'dist',
    // Fuentes woff2 son grandes (~100 KB) — no inlinear en CSS
    assetsInlineLimit: (filePath) => {
      if (/\.(woff2?|ttf|eot)$/i.test(filePath)) return false;
      return 4096; // 4 KB para otros assets
    },
    rollupOptions: {
      output: {
        assetFileNames: (assetInfo) => {
          if (/\.(woff2?|ttf|eot|otf)$/i.test(assetInfo.names?.[0] ?? '')) {
            return 'assets/fonts/[name]-[hash][extname]';
          }
          return 'assets/[name]-[hash][extname]';
        },
      },
    },
  },
});
```

**`frontend/index.html`:**
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

**`frontend/src/main.js`:**
```javascript
import { mount } from 'svelte';
import App from './App.svelte';

const app = mount(App, { target: document.getElementById('app') });
export default app;
```

**Nota:** En Svelte 5, `mount()` reemplaza a `new App({ target })`. La API legacy `new App()` aún funciona en Svelte 5 por compatibilidad pero se desaconseja.

**Confianza:** HIGH — confirmado con npm view y documentación oficial de Svelte 5.

---

### Patrón 3: Svelte 5 Runes — reactivity pattern para esta SPA

**Estado global (stores compartidos entre componentes):**

En Svelte 5, el patrón de estado global para SPAs sin SSR es un módulo `.svelte.js` (extension especial) con runes `$state`. Este módulo es importable desde cualquier componente y mantiene la reactividad.

**`frontend/src/stores/app.svelte.js`:**
```javascript
// Estado global de la SPA — importar con: import { appState } from '../stores/app.svelte.js';

export const appState = $state({
  activeTab: 'cifrar',       // 'cifrar' | 'descifrar' | 'historial'
  historyEnabled: false,      // sincronizado con localStorage 'bed.historyEnabled'
  theme: 'auto',              // 'light' | 'dark' | 'auto' — localStorage 'bed.theme'
});

// Inicializar desde localStorage al cargar
export function initFromStorage() {
  const h = localStorage.getItem('bed.historyEnabled');
  appState.historyEnabled = h === 'true';
  const t = localStorage.getItem('bed.theme');
  if (t === 'light' || t === 'dark' || t === 'auto') appState.theme = t;
}

export function setTheme(theme) {
  appState.theme = theme;
  localStorage.setItem('bed.theme', theme);
}

export function setHistoryEnabled(enabled) {
  appState.historyEnabled = enabled;
  localStorage.setItem('bed.historyEnabled', String(enabled));
  if (!enabled && appState.activeTab === 'historial') {
    appState.activeTab = 'cifrar';
  }
}
```

**Dentro de componentes:**
```svelte
<script>
  import { appState, setHistoryEnabled } from '../stores/app.svelte.js';
  // $state rune declarado en el módulo — reactivo directamente
  // $derived para valores computados
  const isHistorialVisible = $derived(appState.historyEnabled);
</script>

<button
  role="switch"
  aria-checked={appState.historyEnabled}
  onclick={() => setHistoryEnabled(!appState.historyEnabled)}
>
  Historial
</button>
```

**Anti-patrón a evitar:** Usar `writable` stores de Svelte 4 (`import { writable } from 'svelte/store'`). Aunque siguen funcionando en Svelte 5, la forma idiomática son los módulos `.svelte.js` con `$state`. El Migration Guide de Svelte 5 documenta ambos, pero los stores legacy no deben usarse en código nuevo.

**Confianza:** HIGH — documentación oficial svelte.dev + mainmatter.com blog (marzo 2025).

---

### Patrón 4: History endpoints Rust — directory scan

**Confirmación de decisión:** D-27 en CONTEXT.md documenta explícitamente que `GET /api/history` usa directory scan de `/data/encrypted/`. No se usa `redb` en Phase 2.

**Path configurable vía env var (obligatorio para tests):**

```rust
// crates/server/src/state.rs (añadir función)
pub fn data_dir() -> std::path::PathBuf {
    std::env::var("BED_DATA_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/data/encrypted"))
}
```

**Formato de nombre de archivo:** `<timestamp-iso>-<short-id-8chars>.bed`
- Timestamp: `%Y%m%dT%H%M%SZ` (UTC, sin guiones ni colons, sortable alfabéticamente) o ISO 8601 `2026-01-15T14:30:00Z`. Se recomienda formato sin puntuación para evitar caracteres problemáticos en filesystems: `20260115T143000Z-a3f7b2c1.bed`.
- Short-id: 8 chars hex lowercase de `uuid::Uuid::new_v4().simple().to_string()[..8]`.

**Validación de ID para anti path traversal (D-29):**
```rust
fn validate_id(id: &str) -> bool {
    id.len() == 8 && id.chars().all(|c| c.is_ascii_hexdigit() && c.is_lowercase())
}
```

**Implementación `GET /api/history`:**
```rust
use tokio::fs;
use std::path::Path;

pub struct HistoryEntry {
    pub id: String,
    pub timestamp: String,  // ISO 8601 legible
    pub filename: String,
    pub size_bytes: u64,
}

pub async fn list_entries(data_dir: &Path) -> Result<Vec<HistoryEntry>, AppError> {
    let mut dir = fs::read_dir(data_dir).await.map_err(|_| AppError::Internal)?;
    let mut entries = Vec::new();
    while let Some(entry) = dir.next_entry().await.map_err(|_| AppError::Internal)? {
        let name = entry.file_name().to_string_lossy().to_string();
        // Parsear "20260115T143000Z-a3f7b2c1.bed"
        if let Some(parsed) = parse_filename(&name) {
            let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
            entries.push(HistoryEntry { size_bytes: size, ..parsed });
        }
    }
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // desc
    Ok(entries)
}
```

**`POST /api/history` — escritura del .bed:**
```rust
// Request: { "bed_b64": "..." }
// 1. Validar base64 (base64::decode)
// 2. Generar id = uuid v4 truncado a 8 chars hex
// 3. Generar filename = "{timestamp}-{id}.bed"
// 4. Escribir bytes en data_dir/filename
// 5. Return { "id": "...", "timestamp": "...", "filename": "..." }
```

**`GET /api/history/:id` — regenerar armored + QR desde .bed:**
```rust
// 1. Validar id (8 chars hex lowercase)
// 2. Buscar archivo con glob data_dir/*-{id}.bed
// 3. Leer bytes del .bed
// 4. bed_b64 = base64::encode(&bytes)
// 5. armored = bed_core::encode_armored(&bytes)  ← función ya existe en core
// 6. qr_png = bed_core::generate_qr(&armored)?   ← función ya existe en core
// 7. Return { bed_b64, armored, qr_png_b64 }
```

**`DELETE /api/history/:id`:**
```rust
// 1. Validar id
// 2. Buscar archivo, 404 si no existe
// 3. tokio::fs::remove_file(path).await
// 4. Return 204 No Content
```

**Confianza:** HIGH — patrones confirmados en código Phase 1 existente + tokio::fs docs.

---

### Patrón 5: Entrega de resultados cifrar — triple output simultáneo

**Qué devuelve `POST /api/encrypt` (ya implementado en Phase 1):**
```json
{
  "bed_b64": "<base64 del .bed binario>",
  "armored": "-----BEGIN BITCOIN ENCRYPTED BACKUP-----\n...\n-----END BITCOIN ENCRYPTED BACKUP-----\n",
  "qr_png_b64": "<base64 del PNG>"
}
```

**Cómo lo consume el frontend (D-09):**

```svelte
<script>
  let result = $state(null);   // null o { bed_b64, armored, qr_png_b64 }
  let loading = $state(false);
  let error = $state(null);

  async function handleCifrar() {
    loading = true; error = null;
    try {
      const resp = await fetch('/api/encrypt', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ descriptor: descriptorText }),
      });
      if (!resp.ok) {
        const data = await resp.json();
        error = data.error?.message ?? 'Error desconocido';
        return;
      }
      result = await resp.json();
      // Si history toggle ON, llamar POST /api/history (D-12)
      if (appState.historyEnabled) {
        await saveToHistory(result.bed_b64);
      }
    } catch {
      error = 'No se pudo conectar al servidor local.';
    } finally {
      loading = false;
    }
  }

  function downloadBed() {
    const bytes = Uint8Array.from(atob(result.bed_b64), c => c.charCodeAt(0));
    const blob = new Blob([bytes], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'backup.bed'; a.click();
    URL.revokeObjectURL(url);
  }
</script>
```

**QR PNG inline:**
```svelte
{#if result}
  <img src="data:image/png;base64,{result.qr_png_b64}" alt="QR del backup cifrado" />
{/if}
```

**Confianza:** HIGH — pattern directo de las APIs web estándar + contrato del backend ya implementado.

---

### Patrón 6: Descifrar — detección automática armored vs binario

El backend (`crates/server/src/routes/decrypt.rs`) ya implementa la detección automática por bytes mágicos `b"-----BEGIN"`. El frontend solo necesita enviar el contenido tal cual vía multipart:

```javascript
// Auto-detección en frontend: si el texto en textarea empieza con "-----BEGIN"
// → enviarlo como texto en el campo "bed" del multipart
// Si es un File object (de file picker o drop-zone) → enviarlo como binario

async function handleDescifrar() {
  const formData = new FormData();
  if (bedFile) {
    formData.append('bed', bedFile);           // File object
  } else if (armoredText.trim().startsWith('-----BEGIN')) {
    formData.append('bed', new Blob([armoredText], { type: 'text/plain' }), 'armored.txt');
  }
  formData.append('xpub', xpubText.trim());
  // fetch POST /api/decrypt con formData (multipart automático)
}
```

**Drop-zone accesible (D-13):**
```svelte
<div
  role="button"
  tabindex="0"
  ondragover={(e) => { e.preventDefault(); dragOver = true; }}
  ondragleave={() => { dragOver = false; }}
  ondrop={(e) => { e.preventDefault(); dragOver = false; handleDrop(e.dataTransfer.files[0]); }}
  onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') fileInput.click(); }}
  aria-label="Zona de carga. Arrastra un archivo .bed aquí o presiona Enter para seleccionar."
>
  <!-- Texto del drop-zone -->
</div>
<input type="file" bind:this={fileInput} accept=".bed" style="display:none"
  onchange={(e) => handleDrop(e.target.files[0])} />
```

**Confianza:** HIGH — HTML5 Drag and Drop API + código existente de detect armored en decrypt.rs.

---

### Patrón 7: Clipboard API — fallback para contexto no-HTTPS

**El problema crítico:** `navigator.clipboard.writeText()` requiere secure context (HTTPS o localhost). La app se accede por Tor onion HTTP o LAN `.local` — ambos pueden ser no-HTTPS desde el punto de vista del navegador.

**Situación real:**
- Tor onion (`.onion`): tratado como secure context por Firefox. Chrome lo trata como insecure.
- LAN `.local` con HTTPS (StartOS termina TLS): secure context. Con HTTP plain: insecure.
- StartOS 0.4.0: interface LAN tiene `ssl: false` en el manifest — StartOS termina TLS por fuera, pero la conexión interna es HTTP. El navegador ve HTTPS en la URL `.local` → secure context.

**Conclusión:** El dominio `.onion` en Firefox es secure context; en Chromium no. Para máxima compatibilidad, implementar fallback:

```javascript
async function copyToClipboard(text) {
  if (navigator.clipboard && window.isSecureContext) {
    await navigator.clipboard.writeText(text);
  } else {
    // Fallback: execCommand('copy') — deprecated pero ampliamente soportado
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.position = 'fixed'; textarea.style.opacity = '0';
    document.body.appendChild(textarea);
    textarea.focus(); textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
  }
}
```

**Confianza:** HIGH — confirmado en MDN + múltiples fuentes (2025).

---

### Patrón 8: BBQR en Descifrar tab (D-15)

**Cuándo se usa BBQR:** Solo en la tab Descifrar, para el "Mostrar QR" del descriptor recuperado. Si el descriptor en claro cabe en un QR estándar ECC-L (~500 bytes de texto alfanumérico, ~1900 bytes en modo binario), se muestra un QR estático. Si excede, se usa BBQR animado para que Sparrow/Nunchuk puedan escanearlo.

**Sparrow (confirmado):** Soporta BBQr desde v1.9.1 (julio 2024). Acepta output descriptors vía QR.
**Nunchuk (confirmado):** Usa BC-UR2 preferentemente, pero también acepta output descriptors en texto plano QR. El soporte de BBQr en Nunchuk no está confirmado con certeza — se debe mostrar siempre la opción de copiar/descargar el descriptor como texto y usar el QR como complemento opcional.

**Integración BBQR (solo Descifrar tab, lazy import):**

```svelte
<script>
  let showQr = $state(false);
  let qrParts = $state([]);
  let qrCurrentFrame = $state(0);
  let qrInterval = null;

  async function handleMostrarQR(descriptor) {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(descriptor);
    // QR estático si cabe (~700 bytes para ECC-L con modo byte)
    if (bytes.length <= 700) {
      // Usar una librería QR simple cliente-side o canvas
      // Alternativa: pedir QR al backend (POST /api/qr con descriptor)
    } else {
      // BBQR animado — import dinámico para no añadir al bundle principal
      const { splitQRs } = await import('bbqr');
      const { parts } = splitQRs(bytes, 'U', { minSplit: 2, maxVersion: 40 });
      qrParts = parts;
      // Animar ciclo de frames
      qrInterval = setInterval(() => {
        qrCurrentFrame = (qrCurrentFrame + 1) % qrParts.length;
      }, 250);
    }
    showQr = true;
  }
</script>
```

**Tamaño `bbqr@1.2.0`:** El tarball completo es 123.5 KB / 436.6 KB descomprimido. Con import dinámico (`import('bbqr')`), Vite lo pone en un chunk separado que no se carga hasta que el usuario hace click en "Mostrar QR". Esto no impacta el bundle inicial de 50 KB.

**Renderizado QR en browser:** `bbqr` devuelve strings (los partes del QR como texto en formato BBQr). Para renderizar esos strings como imágenes QR, se necesita una librería QR de canvas. Opciones:
- `qrcode` npm (~8 KB minified): estándar, genera canvas o SVG.
- `qrcode-generator` (~4 KB): más ligero, API de bajo nivel.

**Recomendación:** Incluir `qrcode` (npm) para renderizar los frames BBQR en canvas/img. Con import dinámico también, solo se carga cuando se necesita. El bundle inicial no se ve afectado.

**Confianza:** MEDIUM — BBQr confirmado en npm; integración específica con Nunchuk no totalmente verificada (LOW para Nunchuk specifically).

---

### Patrón 9: Fuentes self-hosted — @font-face en CSS

**Fuentes requeridas (D-02, D-43):**
- Inter variable — descargar de https://rsms.me/inter/ (woff2 variable)
- JetBrains Mono variable — descargar de https://www.jetbrains.com/lp/mono/ (woff2 variable)

**En `frontend/src/app.css` o dentro del componente raíz:**
```css
@font-face {
  font-family: 'Inter';
  src: url('/src/assets/fonts/Inter.woff2') format('woff2');
  font-weight: 100 900;    /* variable weight range */
  font-style: normal;
  font-display: swap;
}

@font-face {
  font-family: 'JetBrains Mono';
  src: url('/src/assets/fonts/JetBrainsMono.woff2') format('woff2');
  font-weight: 100 800;
  font-style: normal;
  font-display: swap;
}
```

**Cómo Vite las procesa:** Con el `assetsInlineLimit` configurado para excluir `.woff2`, Vite copia las fuentes a `dist/assets/fonts/[name]-[hash].woff2` y reescribe las URLs en el CSS generado. `rust-embed` las incluye automáticamente al embeder `frontend/dist/`. Resultado: cero requests externos.

**Verificación:** En DevTools Network panel, verificar que tras el initial load no aparezca ninguna request a dominio externo. Hacerlo con `vite preview` que simula producción.

**Confianza:** HIGH — patrón estándar de Vite, confirmado en docs oficiales.

---

### Patrón 10: AppError — variantes nuevas para history

Añadir a `crates/server/src/error.rs`:

```rust
#[error("Entrada de historial no encontrada.")]
HistoryNotFound,

#[error("No se pudo escribir en el historial.")]
HistoryWriteFailed,

#[error("ID de historial inválido.")]
HistoryInvalidId,
```

Mapeo en `IntoResponse`:
- `HistoryNotFound` → 404 `HISTORY_NOT_FOUND`
- `HistoryWriteFailed` → 500 `HISTORY_WRITE_FAILED`
- `HistoryInvalidId` → 422 `HISTORY_INVALID_ID`

**Confianza:** HIGH — patrón ya establecido en Phase 1.

---

## No construir manualmente

| Problema | No construir | Usar en cambio | Por qué |
|----------|-------------|----------------|---------|
| MIME types para assets embebidos | Mapa manual de extensiones | `mime_guess` (transitiva de rust-embed axum-ex) | Cubre 1000+ tipos MIME incluyendo woff2 |
| BBQR encoded QR | Implementación propia del spec | `bbqr@1.2.0` (Coinkite, Public Domain) | El spec tiene sutilezas en el encoding y la corrección de errores QR |
| Renderizado QR a PNG/canvas | Canvas manual con módulos QR de bajo nivel | `qrcode` npm (o `qrcode-generator`) | Maneja versiones QR, corrección de errores, encoding automático |
| Clipboard copy con fallback | Código adhoc | Función utilitaria con `navigator.clipboard` + `execCommand` fallback | Los edge cases son numerosos (permisos, secure context, Safari) |
| UUID generación | RNG manual + hex encoding | `uuid::Uuid::new_v4()` (ya en workspace) | CSPRNG correcto, formato estándar |
| Anti path traversal de `id` | Regex compleja | Validación simple `len == 8 && all hex lowercase` + `fs::read_dir` + búsqueda por nombre | Suficiente para el formato `[a-z0-9]{8}` |

**Insight clave:** La mayor fuente de bugs en esta fase no será la criptografía (Phase 1 la resuelve) sino el clipboard, el manejo de archivos en el browser y el pipeline de build. Mantener los handlers de estos problemas delgados y bien encapsulados.

---

## Trampas comunes

### Trampa 1: `rust-embed` busca `frontend/dist/` en tiempo de compilación — no en runtime

**Qué sale mal:** Si se ejecuta `cargo build` antes de `npm run build`, la carpeta `frontend/dist/` no existe y el build Rust falla con un error críptico del macro `RustEmbed`.

**Por qué ocurre:** `#[derive(RustEmbed)]` resuelve la ruta del folder en compile time. Si el directorio no existe, error.

**Cómo evitar:** En el Dockerfile (Phase 3), el build Node siempre precede al build Rust. En desarrollo local, documentar en README que `npm run build` debe ejecutarse antes que `cargo build` (o usar `cargo watch` que se recompila tras cambios en `dist/`). Añadir al Makefile / script de dev: `cd frontend && npm run build && cd .. && cargo run`.

**Señales de alerta:** Error en `proc_macro` durante `cargo build` mencionando el folder path.

---

### Trampa 2: Fuentes woff2 inlineadas en CSS → supera el límite de 50 KB

**Qué sale mal:** Sin configurar `assetsInlineLimit`, Vite podría intentar inlinear fuentes pequeñas (si la variable woff2 tiene un subset pequeño). Las fuentes completas Inter y JetBrains Mono son ~200 KB cada una — definitivamente fuera del límite si se inlinean.

**Cómo evitar:** Configurar `assetsInlineLimit` como callback que retorna `false` para `.woff2`. Ver Patrón 2 arriba.

**Señales de alerta:** CSS generado en `dist/` con `url(data:font/woff2;base64,...)` de miles de caracteres.

---

### Trampa 3: `navigator.clipboard` undefined en Tor Browser / Chromium HTTP

**Qué sale mal:** El botón "Copiar al portapapeles" no funciona silenciosamente. El usuario no recibe feedback.

**Cómo evitar:** Usar la función `copyToClipboard()` con fallback `execCommand` del Patrón 7. El toast y el label change del botón (D-34) se deben mostrar siempre que la operación se complete (independientemente del path usado).

**Señales de alerta:** `navigator.clipboard` es `undefined` en la consola del navegador en contexto HTTP no-localhost.

---

### Trampa 4: Path traversal en `DELETE /api/history/:id`

**Qué sale mal:** Si `id` no se valida estrictamente, un atacante podría construir un ID con `../` o similar para borrar archivos fuera de `/data/encrypted/`.

**Cómo evitar:** Validar que `id` matchea exactamente `[a-z0-9]{8}` antes de construir el path. Usar `data_dir.join(format!("*-{id}.bed"))` y verificar que el archivo resultante esté dentro de `data_dir` con `path.starts_with(&data_dir)`.

**Señales de alerta:** El test integration que pasa `../../etc/passwd` como id debería retornar 422.

---

### Trampa 5: Descriptor recuperado persiste en memoria tras navegar entre tabs

**Qué sale mal:** Si el estado del descriptor en Descifrar se guarda en el estado global (appState), persiste al cambiar de tab, violando D-16.

**Cómo evitar:** El resultado del descifrado (`descriptor`, `xpub`) deben ser estado LOCAL del componente `TabDescifrar.svelte` (variables `$state` dentro del `<script>` del componente), no en el store global. Al desmontar el componente (cuando se cambia de tab), Svelte descarta automáticamente el estado local.

**Señales de alerta:** El descriptor sigue visible cuando el usuario regresa a Tab Descifrar después de haberla abandonado.

---

### Trampa 6: Ruta `/{*file}` en axum captura rutas `/api/*`

**Qué sale mal:** Si el wildcard `/{*file}` se declara antes de las rutas API específicas, y axum procesa primero el wildcard, las llamadas a `/api/encrypt` serían tratadas como assets estáticos.

**Cómo evitar:** En axum, los matchers más específicos tienen precedencia sobre wildcards. La ruta `/api/encrypt` es más específica que `/{*file}`. Sin embargo, para mayor claridad, declarar las rutas API con prefijo `/api` y las rutas de assets por separado (o usar `.nest("/api", api_router)`).

**Señales de alerta:** `POST /api/encrypt` retorna 404 cuando la ruta `/{*file}` está declarada.

---

### Trampa 7: `BED_DATA_DIR` no configurada en tests — escribe en `/data/encrypted/` real

**Qué sale mal:** Los tests de integración para los history handlers crean archivos reales en `/data/encrypted/` que no se limpian, contamina el sistema y potencialmente falla en CI donde ese directorio no existe.

**Cómo evitar:** En todos los tests de history, usar `tempfile::tempdir()` y establecer `std::env::set_var("BED_DATA_DIR", tmpdir.path())` antes de llamar al handler. El accessor `data_dir()` lo leerá correctamente.

**Señales de alerta:** Los tests de history fallan en CI con "No such file or directory: /data/encrypted".

---

## Ejemplos de código verificados

### Cifrar result: blob download del .bed

```javascript
// Source: Web standard FileAPI + atob() — MDN
function triggerBedDownload(bed_b64) {
  const binary = atob(bed_b64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  const blob = new Blob([bytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `backup-${Date.now()}.bed`;
  link.click();
  URL.revokeObjectURL(url);
}
```

### QR PNG inline en Svelte 5

```svelte
<!-- Source: data URI standard — MDN -->
{#if result?.qr_png_b64}
  <figure>
    <img
      src="data:image/png;base64,{result.qr_png_b64}"
      alt="Código QR del backup cifrado"
      width="200"
      height="200"
    />
    <figcaption>QR del backup cifrado</figcaption>
  </figure>
{/if}
```

### redb pattern (referencia para v1.x, NO Phase 2)

```rust
// Source: docs.rs/redb/latest — para referencia futura si se añade redb
use redb::{Database, TableDefinition};
const HISTORY: TableDefinition<&str, &str> = TableDefinition::new("history");

let db = Database::create(data_dir.join("index.redb"))?;
let write_txn = db.begin_write()?;
{
    let mut table = write_txn.open_table(HISTORY)?;
    table.insert(id.as_str(), filename.as_str())?;
}
write_txn.commit()?;
```

### AppError nuevas variantes — IntoResponse match

```rust
// Añadir al match en IntoResponse for AppError:
AppError::HistoryNotFound => (StatusCode::NOT_FOUND, "HISTORY_NOT_FOUND"),
AppError::HistoryWriteFailed => (StatusCode::INTERNAL_SERVER_ERROR, "HISTORY_WRITE_FAILED"),
AppError::HistoryInvalidId => (StatusCode::UNPROCESSABLE_ENTITY, "HISTORY_INVALID_ID"),
```

### Validación xpub cliente-side (D-14)

```javascript
// Source: D-14 CONTEXT.md — regex de validación cliente
const XPUB_RE = /^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$/;
const isXpubValid = (s) => XPUB_RE.test(s.trim());
```

---

## Disponibilidad de entorno

| Dependencia | Requerida por | Disponible | Versión | Fallback |
|-------------|---------------|------------|---------|----------|
| Node.js | Build frontend Vite | ✓ | 20.20.1 (LTS) | — |
| npm | Instalar deps frontend | ✓ | 10.8.2 | — |
| cargo/rustc | Build binario Rust | ✓ | 1.93.0 (stable) | — |
| `frontend/dist/` | rust-embed en compile time | ✗ (aún no creado) | — | Ejecutar `npm run build` primero |
| `/data/encrypted/` | History endpoints (runtime) | ✗ (solo en StartOS) | — | `BED_DATA_DIR=./data/encrypted` en dev; `tempfile::tempdir()` en tests |
| Inter woff2 | UI fonts | ✗ (pendiente descargar) | N/A | system-ui fallback (D-02) |
| JetBrains Mono woff2 | UI monoespaciado | ✗ (pendiente descargar) | N/A | Courier New fallback (D-02) |

**Dependencias faltantes sin fallback en producción:**
- `frontend/dist/` — se crea con `npm run build` como Wave 0 del plan.
- Fuentes woff2 — se descargan como Wave 0.

**Dependencias faltantes con fallback viable:**
- `/data/encrypted/` — en dev y tests, configurar con `BED_DATA_DIR`.

---

## Estado del arte

| Enfoque antiguo | Enfoque actual (2026) | Cuándo cambió | Impacto |
|----------------|-----------------------|----------------|---------|
| Svelte 4 `writable` stores | Svelte 5 `.svelte.js` módulos con `$state` | Oct 2024 (Svelte 5 GA) | No usar `writable` en código nuevo; stores legacy siguen funcionando |
| `new App({ target })` en main.js | `mount(App, { target })` en main.js | Oct 2024 (Svelte 5 GA) | API legacy compatible pero deprecada |
| `tower-http::ServeDir` para assets | `rust-embed` con `axum-ex` | 2023+ (rust-embed 8.x) | ServeDir lee del FS en runtime (necesita mount en container); rust-embed bake en binario |
| `sled` para KV embebido | `redb` 4.x o directory scan | 2021 (sled abandono) | sled produce memory spikes de 5 GB+, nunca alcanzó estable |
| Single QR para todo | BBQR animado para payloads grandes | 2023 (Coldcard/Coinkite) | Sparrow 1.9.1+ lo soporta para descriptors; Nunchuk usa BC-UR2 preferentemente |

**Deprecado/obsoleto:**
- `document.execCommand('copy')`: deprecated en spec pero aún soportado como fallback. No usar como path primario.
- `$: reactive` statements en Svelte 5: funcionan (modo legacy) pero reemplazar con `$derived`.

---

## Preguntas abiertas

1. **Soporte BBQr en Nunchuk para descriptor import**
   - Lo que sabemos: Nunchuk usa BC-UR2 principalmente. Soporta "Output Descriptor Import" pero no queda claro si acepta BBQR específicamente.
   - Lo que no está claro: Si el usuario escanea un QR BBQR desde BED en la app Nunchuk, ¿lo reconoce como descriptor?
   - Recomendación: No bloquear Phase 2 por esto. Documentar en la UI que el QR BBQR está optimizado para Sparrow/Coldcard. Nunchuk puede usar la opción "Copiar texto" o "Descargar .txt" para importar.

2. **Peso real del bundle de 50 KB con Inter + JetBrains Mono**
   - Lo que sabemos: Las fuentes woff2 variable son ~200-300 KB cada una. El límite de 50 KB de CLAUDE.md específica "SPA bundle", y UI-SPEC.md confirma "fonts excluded from this count".
   - Lo que no está claro: ¿Se verifica el límite de 50 KB solo sobre JS/CSS, o incluye fuentes?
   - Recomendación: Seguir UI-SPEC.md: 50 KB aplica a JS+CSS, fuentes excluidas. Documentar en el build que las fuentes se sirven como assets separados.

3. **Usar `qrcode` npm vs otro para renderizar QR en Descifrar tab**
   - Lo que sabemos: `bbqr` devuelve los datos del QR como strings de texto (partes del BBQr). Para renderizarlos en pantalla se necesita un renderer QR separado.
   - Recomendación: Añadir `qrcode` npm (o `qrcode-generator`) solo en el chunk dinámico de BBQR para no impactar el bundle inicial. Decisión final al planner.

---

## Fuentes

### Primarias (confianza HIGH)
- [docs.rs/crate/rust-embed/latest](https://docs.rs/crate/rust-embed/latest) — feature `axum-ex`, ejemplo axum, patrón de embedding
- [docs.rs/crate/rust-embed/latest/source/examples/axum.rs](https://docs.rs/crate/rust-embed/latest/source/examples/axum.rs) — código ejemplo oficial
- [svelte.dev/docs/svelte/getting-started](https://svelte.dev/docs/svelte/getting-started) — scaffold Svelte 5 + Vite plain
- [svelte.dev/docs/svelte/$state](https://svelte.dev/docs/svelte/$state) — rune $state, módulos .svelte.js
- [svelte.dev/docs/svelte/$derived](https://svelte.dev/docs/svelte/$derived) — rune $derived
- [svelte.dev/docs/svelte/v5-migration-guide](https://svelte.dev/docs/svelte/v5-migration-guide) — cambios breaking Svelte 5
- [docs.rs/redb/latest](https://docs.rs/redb) — TableDefinition API, write transactions
- [github.com/coinkite/BBQr](https://github.com/coinkite/BBQr) — spec + implementación JS
- `npm view bbqr version` → 1.2.0 (verificado 2026-05-06)
- `npm view svelte version` → 5.55.5 (verificado 2026-05-06)
- `npm view vite version` → 8.0.10 (verificado 2026-05-06)
- `npm view @sveltejs/vite-plugin-svelte version` → 7.1.1 (verificado 2026-05-06)
- [vite.dev/config/build-options](https://vite.dev/config/build-options) — assetsInlineLimit callback, assetFileNames
- [developer.mozilla.org/en-US/docs/Web/API/Clipboard_API](https://developer.mozilla.org/en-US/docs/Web/API/Clipboard_API) — secure context requirement
- Código existente Phase 1 en `crates/server/src/` (leído directamente)

### Secundarias (confianza MEDIUM)
- [mainmatter.com/blog/2025/03/11/global-state-in-svelte-5/](https://mainmatter.com/blog/2025/03/11/global-state-in-svelte-5/) — patrones de estado global Svelte 5
- [dev.to/johalputt — caso estudio Svelte 5 + Vite 6](https://dev.to/johalputt/case-study-we-cut-bundle-size-by-45-using-svelte-50-and-vite-60-10jp) — bundle size reducción
- [sparrowwallet.net/sparrow-wallet-release-1-9-1/](https://sparrowwallet.net/sparrow-wallet-release-1-9-1/) — confirma BBQr support en Sparrow 1.9.1+
- [docs.rs/axum-embed/latest/axum_embed/](https://docs.rs/axum-embed/latest/axum_embed/) — alternativa `axum-embed` crate (axum 0.7, no 0.8)

### Terciarias (confianza LOW — pendiente de validación)
- Soporte BBQr en Nunchuk — no verificado con fuente primaria oficial de Nunchuk docs.

---

## Metadatos de confianza

| Área | Nivel | Razón |
|------|-------|-------|
| Stack Svelte 5 + Vite 6 | HIGH | Versiones verificadas con npm view; docs oficiales consultadas |
| rust-embed + axum | HIGH | Ejemplo oficial en docs.rs; feature axum-ex confirmada para axum ^0.8 |
| History endpoints Rust | HIGH | Patrón establecido en Phase 1; tokio::fs es estándar |
| Directory scan vs redb | HIGH | D-27 en CONTEXT.md lo documenta explícitamente |
| BBQR en browser | MEDIUM | Paquete verificado; integración con Nunchuk no totalmente confirmada |
| Clipboard API fallback | HIGH | MDN + múltiples fuentes de 2025 |
| Font embedding Vite | HIGH | Patrón assetFileNames confirmado en Vite docs + PR |
| Bundle size <50 KB | MEDIUM | El preset Svelte 5 + Vite produce bundles pequeños; verificar en Wave final con análisis de bundle |

**Fecha de investigación:** 2026-05-06
**Válido hasta:** 2026-06-05 (stack estable; fuentes woff2 y bbqr pueden actualizar)
