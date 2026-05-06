# Phase 2: SPA Frontend + History - Context

**Gathered:** 2026-05-06
**Status:** Ready for UI design contract (`/gsd:ui-phase 2`) → planning

<domain>
## Phase Boundary

Phase 2 entrega la SPA Svelte 5 + Vite 6 que un holder StartOS abre en el navegador (Tor onion o LAN `.local`) y usa para cifrar descriptors multisig en los tres formatos (`.bed` binario, armored, QR) y para descifrarlos con cualquier xpub cosigner. Todos los assets se sirven desde el binario Rust vía `rust-embed` (sin CDN, sin fonts remotas, sin telemetría). Phase 2 también añade el toggle opt-in de modo histórico que persiste el `.bed` cifrado en `/data/encrypted/` y los endpoints HTTP nuevos para listar y borrar entradas. El descriptor en claro nunca toca disco — ni en tránsito ni en historial. Visible en la UI: modelo de amenazas resumido (UI-03).

**Fuera de Phase 2:**
- Persistencia del toggle history backend cross-restart (PERS-01 → v1.x con `file-models`).
- Drag-and-drop avanzado, "test decrypt" automático, checksum visual del descriptor recuperado, errores específicos por sintaxis del descriptor (todos UX2-* → v2).
- Camera scanner QR (Out of Scope: HTTPS secure-context requirement complica acceso por Tor/LAN).
- Cualquier cambio breaking al contract de `/api/encrypt` o `/api/decrypt` (Phase 1 stable — solo extensiones aditivas no-breaking permitidas).
- Dockerfile, GHCR, s9pk (Phase 3 + Phase 4).

</domain>

<decisions>
## Implementation Decisions

### Estética & Tipografía
- **D-01:** Look profesional minimal cálido, no industrial-frío. Mobile-first responsive (breakpoints típicos 360 / 768 / 1024+). Touch-friendly (mínimo 44×44 px en botones).
- **D-02:** Tipografía bundleada: **Inter** (UI body/headings) + **JetBrains Mono** (descriptors, xpubs, IDs hex). Self-hosted como woff2 variable, embebido vía Vite + `rust-embed`. **NO Google Fonts CDN** (viola constraint sin-CDN). Fallback `system-ui, -apple-system, Segoe UI, Roboto, sans-serif` mientras carga.
- **D-03:** Modo claro / oscuro / auto siguiendo `prefers-color-scheme`. Toggle manual en header (3 estados: light / dark / auto). Persiste preferencia en `localStorage` con clave `bed.theme`.
- **D-04:** Paleta y spacing scale concretos = **Claude's discretion** durante UI-SPEC. Restricción: contraste mínimo WCAG AA 4.5:1 texto normal, 3:1 grande, 3:1 componentes UI.

### Layout & Navegación
- **D-05:** Estructura: header global + área principal con 2 ó 3 tabs según toggle.
  - Header: logo/título "BED" + toggle modo histórico + toggle theme + link "Modelo de amenazas".
  - Tabs fijas: **Cifrar** y **Descifrar**.
  - Tab condicional: **Historial** (visible solo si toggle modo histórico = ON; oculta o disabled si OFF).
- **D-06:** Routing **state-based interno** con stores Svelte. **Sin URL hash routing** en v1. Razón: app local en Tor/LAN, deep-linking a tabs no aporta valor; menos complejidad.
- **D-07:** Tabs implementadas con roles ARIA estándar (`role="tablist" / "tab" / "tabpanel"`, `aria-selected`, `aria-controls`).

### Cifrar — Flujo & Outputs
- **D-08:** Tab Cifrar tiene un único form: textarea para el descriptor (placeholder con ejemplo `wsh(multi(2, [fp/path]xpub.../<0;1>/*, ...))#checksum`) + botón **"Cifrar"**. Validación inline (mensaje del backend tal cual cuando 422).
- **D-09:** Tras click "Cifrar" exitoso, se muestran los **tres outputs** simultáneamente, en una zona de "resultado" debajo del form (no se limpia el form):
  1. **Archivo `.bed`**: botón "Descargar `.bed`" (genera blob desde `bed_b64` y trigger download).
  2. **Texto armored**: bloque `<pre>` con el armored mostrado + botón "Copiar al portapapeles".
  3. **QR PNG**: imagen renderizada inline (`<img src="data:image/png;base64,{qr_png_b64}">`) + botón "Descargar PNG".
- **D-10:** Usuario elige cuál usar (o varios) — **no hay pre-selección de formato**. Razón: para distribuir backups con redundancia conviene tenerlos todos a mano. Coherente con "que se de a elegir en qué formato".
- **D-11:** Si el descriptor cifrado excede capacidad QR (~2900 B armored), Phase 1 responde 422 `QrTooLarge` y el encrypt entero falla. Frontend muestra el mensaje del backend y sugiere descomponer multisig o usar archivo. **Sin BBQR fallback en Cifrar v1** (eso requeriría cambio breaking en `/api/encrypt`).
- **D-12:** Si `save_to_history` toggle = ON al momento de Cifrar exitoso, el frontend hace una segunda llamada `POST /api/history` con el `bed_b64` para persistir. Si esa llamada falla, mostrar warning "Cifrado OK pero no se guardó en historial" sin invalidar el resultado.

### Descifrar — Flujo & Outputs
- **D-13:** Tab Descifrar tiene dos secciones de input visibles desde el inicio (no stepper):
  - **`.bed` o armored**: drop-zone (drag), textarea (paste armored), botón "Subir archivo" (file picker para binario). Auto-detecta por bytes mágicos / presencia de header.
  - **xpub**: textarea (paste) o botón "Subir archivo" para `.txt` con xpub.
- **D-14:** Botón "Descifrar" disabled hasta que ambos inputs tengan contenido válido (validación cliente: `.bed` no vacío, xpub matches regex `^([xyzt]pub|tpub)[A-Za-z0-9]{100,}$`). Cliente no parsea profundamente — eso lo hace el backend.
- **D-15:** Tras descifrado exitoso, se muestra el descriptor recuperado en bloque `<pre>` mono + tres opciones de export **client-side**:
  1. **Copiar texto**: botón "Copiar al portapapeles" del descriptor en claro.
  2. **Descargar .txt**: trigger download de blob `text/plain` con el descriptor.
  3. **Mostrar QR**: QR plano single-frame si descriptor cabe en QR ECC-L; **BBQR animado** (multi-frame) si excede capacidad. Renderizado on-screen para escaneo desde Sparrow / Nunchuk / wallets compatibles. **No download del QR** — se escanea de pantalla (consistente con flujo wallet import).
- **D-16:** El descriptor recuperado **vive solo en memoria del navegador**. Nunca se persiste en `localStorage`, `sessionStorage`, ni history del navegador. Al navegar a otra tab (Cifrar / Historial) o recargar, el descriptor desaparece. Botón explícito "Limpiar resultado" disponible.
- **D-17:** xpub introducida nunca se persiste en `localStorage` ni se loguea (TraceLayer skip_all ya garantiza no-log en backend, Phase 1). Tras descifrado, el campo xpub se limpia automáticamente (security default).

### History — Toggle, Listado, Borrado
- **D-18:** Toggle "Modo histórico" en el header global. **Default first-visit: OFF** (alineado con doctrina "ephemeral by default"). Estado persiste en `localStorage` con clave `bed.historyEnabled`. Badge visible cuando ON ("Modo histórico activo" + icono cerca del toggle).
- **D-19:** Backend **sin estado global** del toggle (sin `AtomicBool` server-side, simplificación respecto a IDEA original). El toggle es **100% client-side preference**. La persistencia sucede solo cuando el cliente llama explícitamente `POST /api/history` tras un cifrado exitoso. Ventaja: API queda stateless, sin global mutable state, restart-safe.
- **D-20:** Tab Historial visible solo si toggle ON. Si OFF, la tab está oculta del UI. Si el usuario activa el toggle, la tab aparece y queda seleccionable.
- **D-21:** Tab Historial muestra lista de entradas vía `GET /api/history`. Cada entrada: timestamp legible (relativo: "hace 3 días" + tooltip absoluto ISO), short-id, botones "Ver" y "Borrar".
- **D-22:** Click en "Ver" abre un panel/modal con los **tres formatos regenerados on-demand** desde el `.bed` persistido:
  - Descarga `.bed` (directo del archivo en `/data/encrypted/`)
  - Armored regenerado server-side al servir esta vista
  - QR PNG regenerado server-side
  Esto justifica la decisión de persistir solo el `.bed` (los otros dos son derivables).
- **D-23:** Click en "Borrar" abre **modal de confirmación** con descripción de la entrada (timestamp + short-id) y botones "Cancelar" / "Borrar" (rojo). Confirmar dispara `DELETE /api/history/:id`. Tras éxito: entrada desaparece de la lista + toast "Entrada borrada".
- **D-24:** Empty state Historial (toggle ON, lista vacía): texto centrado "Aún no hay backups cifrados. Cifra un descriptor con el modo histórico activo para empezar." + icono de archivo. Sin ilustración pesada.

### API Contract — Extensiones Phase 2 (no-breaking)
- **D-25:** Phase 1 endpoints **inalterados** (`POST /api/encrypt` JSON, `POST /api/decrypt` multipart).
- **D-26:** Endpoint nuevo **`POST /api/history`**:
  - Request JSON: `{"bed_b64": "<base64-encoded .bed binary>"}`
  - Response 200 JSON: `{"id": "<short-id>", "timestamp": "<ISO-8601>", "filename": "<timestamp>-<short-id>.bed"}`
  - Side effect: escribe archivo `/data/encrypted/<timestamp>-<short-id>.bed` (binario, decoded del base64).
  - Errores: 422 si bed_b64 no es base64 válido; 500 si write a disk falla.
  - **NO acepta el descriptor cleartext** — solo el `.bed` ya cifrado. Garantiza HIST-03 (no leak por design).
- **D-27:** Endpoint nuevo **`GET /api/history`**:
  - Response 200 JSON: `{"entries": [{"id": "...", "timestamp": "...", "filename": "...", "size_bytes": 123}, ...]}`
  - Implementación: directory scan de `/data/encrypted/`, parse filename `<timestamp>-<short-id>.bed`, ordenar por timestamp desc.
  - Vacío → `{"entries": []}`.
- **D-28:** Endpoint nuevo **`GET /api/history/:id`** (para regenerar armored y QR del .bed persistido):
  - Response 200 JSON: `{"bed_b64": "...", "armored": "...", "qr_png_b64": "..."}`.
  - Mismo contract que la respuesta de `/api/encrypt` pero leyendo del disk en vez de cifrar.
  - 404 si no existe.
- **D-29:** Endpoint nuevo **`DELETE /api/history/:id`**:
  - Response 204 No Content si OK.
  - 404 si no existe.
  - Implementación: validar que `id` matchea formato `[a-z0-9]{8}` (anti path traversal), buscar archivo correspondiente en `/data/encrypted/`, borrar.

### Threat Model UX (UI-03)
- **D-30:** Sección **colapsable** accesible desde un link/botón en el header global ("Modelo de amenazas" o icono ⚠️ + texto). Por defecto **colapsada** para no saturar el layout principal.
- **D-31:** Contenido al expandir, en este orden:
  1. **Callout destacado** (caja con borde y color de advertencia): *"Ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig"*. Es la regla de oro.
  2. **Lo que protege ✓** (verde / positivo): "Tu descriptor cifrado contra un atacante que solo encuentra el `.bed`."
  3. **Lo que NO protege ✗** (ámbar / cuidado): "Compromiso de StartOS durante el cifrado (el descriptor en claro pasa por memoria del proceso). Atacante que ya tiene una xpub de tu multisig."
  4. Link al README del repo para detalle completo (DOC-01 / DOC-02 viven en Phase 4).
- **D-32:** Implementado como `<details>` HTML semántico (a11y free) o componente Svelte equivalente con role correcto (`aria-expanded`).

### Feedback de Operaciones
- **D-33:** **Loading**: spinner inline dentro del botón principal ("Cifrando…", "Descifrando…", "Guardando…") + botón disabled durante la op. **Sin overlay full-page**.
- **D-34:** **Copy al portapapeles**: combinación toast + label change. (a) Toast efímero superior derecha (3s, "Copiado al portapapeles"). (b) Label del botón cambia a "Copiado ✓" por 1.5s antes de volver al label original. Ambos visibles para cubrir casos donde el toast queda fuera del viewport.
- **D-35:** **Errores del API**: alerta inline arriba del form correspondiente (color advertencia, icono ⚠️, mensaje del backend en castellano tal cual). Cerrable manualmente con × o auto-dismiss tras nueva acción.
- **D-36:** **Confirmación de acciones destructivas** (borrar entrada): modal con texto, descripción de entrada, botón "Cancelar" (default focus) + botón "Borrar" (rojo, requiere segundo click). Sin swipe-to-delete (mobile) para reducir borrados accidentales.

### Accesibilidad — WCAG AA Básico
- **D-37:** Mínimos no-negociables:
  - `<label for>` asociado en cada input.
  - HTML semántico: `<header>`, `<main>`, `<nav>`, `<section>`, `<button>` (no `<div onclick>`).
  - Contraste WCAG AA: ≥4.5:1 texto normal, ≥3:1 grande/UI.
  - Focus visible con outline distintivo (no solo cambio de color).
  - Navegación por tab funcional, orden lógico.
  - Mensajes de error asociados al campo con `aria-describedby`.
  - `aria-live="polite"` en el área de status / toasts para anuncios a lectores de pantalla.
  - Iconos sin texto llevan `aria-label`.
  - No depender solo del color para transmitir info (íconos + texto siempre).
- **D-38:** Auditoría manual con keyboard-only navigation y al menos un screen reader (NVDA / VoiceOver) antes de cerrar la fase. CI no audita a11y en v1 (defer si crece la app).

### Build & Embedding (rust-embed pipeline)
- **D-39:** Frontend en directorio top-level `frontend/` del repo: `frontend/package.json`, `frontend/src/`, `frontend/vite.config.js`, `frontend/dist/` (build output).
- **D-40:** Build pipeline: `cd frontend && npm install && npm run build` → emite `frontend/dist/{index.html, assets/*}`. `crates/server` declara `#[derive(RustEmbed)] #[folder = "../../frontend/dist/"]` para servir los assets.
- **D-41:** Vite configurado para emitir hashed asset filenames (cache busting). `index.html` referencia esos hashes; al servir vía `rust-embed`, los nombres son estables dentro de un build.
- **D-42:** Dev mode: Vite dev server en puerto 5173 con proxy `/api` → `127.0.0.1:8080` (axum). Hot-reload del frontend sin recompilar Rust. **Producción:** todo embebido en el binario; Vite no se ejecuta en runtime.
- **D-43:** Fonts (Inter + JetBrains Mono variable woff2) viven en `frontend/src/assets/fonts/` y se importan vía `@font-face` en CSS local. Vite las copia a `dist/assets/` y `rust-embed` las sirve. **Cero requests externos** — verificable con DevTools Network panel.

### Claude's Discretion
- Paleta exacta de colores (light + dark), spacing scale, radii, shadows, transitions.
- Layout micro-decisiones: gap entre secciones, posición exacta del toggle theme/historial.
- Animaciones / transitions (sutiles, no bloqueantes).
- Estructura interna de stores Svelte (un store global vs múltiples specializados).
- Estilo visual del modal de confirmación, toast, alerts.
- Empty state ilustración (texto + icono mínimo, sin SVG ilustración pesada — ya decidido).
- Patron exacto de manejo de errores fetch (try/catch + error store + render condicional).
- Componente vs página: granularidad de componentización dentro de Svelte.

### Folded Todos
None — `gsd-tools todo match-phase 2` retornó 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning
- `.planning/PROJECT.md` — Vision, constraints, key decisions table
- `.planning/REQUIREMENTS.md` — Phase 2 requirements: UI-01..03, HIST-01..06
- `.planning/ROADMAP.md` §"Phase 2: SPA Frontend + History" — Goal + 4 success criteria
- `.planning/STATE.md` — Current position
- `.planning/phases/01-crypto-core-http-api/01-CONTEXT.md` — API contract estable (D-05..07), AppError enum (D-16..17), validación BIP (D-08..09)
- `IDEA.md` — Original brief
- `CLAUDE.md` — Stack lock-in (Svelte 5 + Vite 6 + rust-embed locked) + "What NOT to Use" (no SvelteKit, no CDN, no Google Fonts) + version compatibility matrix

### Research artifacts (este proyecto)
- `.planning/research/SUMMARY.md` — Locked stack table, build order
- `.planning/research/STACK.md` — Frontend stack: Svelte 5 + Vite 6, redb 4 (no usado en Phase 2), `rust-embed` 8 con feature `axum-ex`
- `.planning/research/ARCHITECTURE.md` — System overview
- `.planning/research/FEATURES.md` — Feature scope mapping para Phase 2 (UI-01..03, HIST-01..06)
- `.planning/research/PITFALLS.md` — Pitfall #5 "persist cleartext" aplica directamente a Phase 2

### Phase 1 implementation (lectura local)
- `crates/server/src/routes/encrypt.rs` — Contract de respuesta `{bed_b64, armored, qr_png_b64}` (D-09)
- `crates/server/src/routes/decrypt.rs` — Contract de respuesta `{descriptor}` (D-15)
- `crates/server/src/error.rs` — `AppError` enum (variantes y status codes — Phase 2 añade variantes para history)
- `crates/server/src/main.rs` — Bind addr + tracing setup (Phase 2 añade routes nuevas al router)
- `crates/core/src/armored.rs` — Encoder/decoder armored (Phase 2 reutiliza para regenerar armored del .bed persistido — D-22, D-28)

### External specs (web — usar `WebFetch` cuando se necesite)
- BIP draft PR `bitcoin/bips#1951` — https://github.com/bitcoin/bips/pull/1951 — formato armored exacto
- Svelte 5 docs — https://svelte.dev/docs/svelte/overview — runes (`$state`, `$derived`, `$effect`), no más `writable` stores legacy
- Vite 6 docs — https://vite.dev/ — config para asset hashing, build options
- `rust-embed` `axum-ex` feature — https://github.com/pyrossh/rust-embed — patrón axum integration
- Inter variable woff2 — https://rsms.me/inter/ (descargar self-hosted, no usar via Google Fonts)
- JetBrains Mono — https://www.jetbrains.com/lp/mono/ (descargar self-hosted)
- BBQR spec — https://github.com/coinkite/BBQr — formato animado QR para Sparrow / Nunchuk / Coldcard
- BBQR JS lib (a verificar en research) — `bbqr` npm o equivalente; license, mantenimiento, tamaño bundle
- WCAG 2.1 AA quick ref — https://www.w3.org/WAI/WCAG21/quickref/?currentsidebar=%23col_overview&levels=aa
- Sparrow descriptor import — https://sparrowwallet.com/docs/quick-start.html (verificar formatos QR aceptados)
- Nunchuk multisig import — https://docs.nunchuk.io/ (verificar formatos QR aceptados)

### Memoria del usuario aplicable
- `feedback_verificar_no_inventar.md` — toda afirmación técnica contrastada en fuente primaria (BBQR, Sparrow/Nunchuk QR support deben verificarse, no asumirse)
- `feedback_no_circles.md` — diagnosticar a fondo antes de proponer fixes
- `feedback_no_ui_changes_without_testing.md` — un cambio UI a la vez, probar cada uno
- `feedback_castellano_no_argentino.md` — todos los strings UI en castellano (tú/descarga/coge), no argentino
- `feedback_git_noreply_email.md` — usar `55397917+4rkad@users.noreply.github.com`

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`crates/server/src/routes/encrypt.rs`** — endpoint funcional. Phase 2 lo llama tal cual.
- **`crates/server/src/routes/decrypt.rs`** — endpoint funcional. Phase 2 lo llama tal cual.
- **`crates/core/src/armored.rs`** — encoder/decoder armored. Phase 2 lo reutiliza en el nuevo handler `GET /api/history/:id` para regenerar el armored desde el `.bed` persistido (D-22, D-28).
- **`crates/core/src/encrypt.rs` + `crates/core/src/qr.rs`** — Phase 2 usa qr generator para regenerar QR PNG desde armored al servir `GET /api/history/:id`.
- **`crates/server/src/error.rs`** — `AppError` enum. Phase 2 añade variantes: `HistoryNotFound` (404), `HistoryWriteFailed` (500), `HistoryInvalidId` (422).

### Established Patterns
- **Stack pre-locked en CLAUDE.md** — Svelte 5 plain (no SvelteKit), Vite 6, rust-embed 8 con feature `axum-ex`. Versiones exactas en `.planning/research/STACK.md`.
- **Error pattern** — `AppError` con `IntoResponse` y body JSON `{error: {code, message}}` (D-16..17 Phase 1). Phase 2 mantiene este pattern para handlers nuevos.
- **Tracing pattern** — `#[tracing::instrument(skip_all)]` en todos los handlers; ningún campo sensible en spans. Phase 2 history handlers heredan este patrón (skip_all sobre body que contiene `bed_b64`).
- **Test pattern** — `tower::ServiceExt::oneshot` + `axum::body::to_bytes` (D-23 Phase 1). Phase 2 history handlers integration tests siguen este patrón.

### Integration Points
- **`crates/server/src/main.rs`** — el `Router` se construye aquí. Phase 2 añade 4 rutas nuevas (`POST /api/history`, `GET /api/history`, `GET /api/history/:id`, `DELETE /api/history/:id`) al router existente.
- **`/data/encrypted/`** directorio creado por Phase 4 (s9pk volume `main`). En dev: `mkdir -p ./data/encrypted/`. En tests: `tempfile::tempdir()`. Path configurable vía env var `BED_DATA_DIR` con default `/data/encrypted/` — necesario para que tests no choquen con prod path.
- **rust-embed integration** — `crates/server/src/main.rs` o submódulo `assets.rs` declara `#[derive(RustEmbed)] #[folder = "../../frontend/dist/"]` y monta como `axum::Router::nest_service` o handler custom que sirve estáticos desde el embed. SPA fallback (`/` y rutas no-API) sirve `index.html`.
- **Cargo workspace** — añadir `rust-embed = "8"` con feature `axum-ex` (o equivalente) a `crates/server/Cargo.toml`. Resto del stack ya pinneado.

</code_context>

<specifics>
## Specific Ideas

- **Tipografía mostrada en mockup**: pruebas reales con descriptors largos en JetBrains Mono para confirmar que line-wrapping se ve limpio en mobile (descriptors típicos llevan checksums tipo `#abc123de` al final, importante que no se corten visual confusamente).
- **Test de no-leak extendido a frontend**: en Phase 2 añadir test integration que post a `/api/history`, recibe respuesta, y verifica con `grep` que el descriptor en claro original NO aparece en ningún archivo bajo `/data/encrypted/<test-tempdir>/`. Refuerza HIST-03.
- **localStorage clave-valor concretos** (definir en planner):
  - `bed.theme` ∈ `light` | `dark` | `auto` (default `auto`).
  - `bed.historyEnabled` ∈ `true` | `false` (default `false` first-visit).
- **QR del descriptor recuperado para Sparrow / Nunchuk**: el formato debe ser texto plano del descriptor (BIP-380 con checksum). Verificar en research que ambos wallets aceptan QR plano de descriptor multisig (Nunchuk en particular suele preferir formato propio "wallet config" — confirmar antes de planning). Si Nunchuk requiere export específico, advertir al usuario o documentar el limit.
- **BBQR JS lib**: priorizar libs maintained y small bundle. `bbqr` npm package de Coinkite es candidato — verificar license (MIT/BSD), tamaño minified, si soporta browser sin polyfills. Alternativa: implementar BBQR cliente-side desde la spec (~200 líneas JS), evita dep externa.
- **Aviso clave del threat model en castellano** (D-31): wording final exacto: *"Ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig."* — capitalizar consistente, sin "ningún" cambio.

</specifics>

<deferred>
## Deferred Ideas

- **PERS-01: persistencia cross-restart del toggle history backend** — D-19 elimina el AtomicBool global; el toggle es 100% client-side preference (localStorage). Si en v1.x se quiere recuperar estado server-side (ej. para multi-device sync), introducir `file-models` y endpoint `GET/POST /api/config`. **Resultado:** PERS-01 se simplifica radicalmente — el cliente ya recuerda solo, no hace falta backend persistence.
- **BBQR fallback en Cifrar** (cuando descriptor cifrado excede 2900 B QR) — requiere cambio en API `/api/encrypt` (retornar `qr_png_b64` opcional o array de frames). Fuera de Phase 2; v2 con `/api/v2/encrypt` o flag opt-in.
- **UX2-01 Drag-and-drop avanzado** — D-13 incluye drop-zone básica para `.bed`. Drag-drop multi-archivo, drag-drop entre tabs, drop con preview detallado → v2.
- **UX2-02 "Test decrypt" round-trip transparente** — antes de declarar "cifrado OK", la app intentaría auto-descifrar con la primera xpub disponible para confirmar reversibilidad. Requiere xpub disponible en sesión, lo cual choca con D-17 (xpub no se persiste). Defer.
- **UX2-03 Display de checksum del descriptor recuperado** — calcular y mostrar checksum BIP-380 del descriptor en claro tras descifrado, para verificación visual. Útil pero no crítico v1.
- **UX2-04 Mensajes específicos por sintaxis del descriptor** — Phase 2 muestra mensaje genérico del backend (`DescriptorParse`). Mensajes específicos (xpub mal-formada, paréntesis sin cerrar, etc.) requieren parser ad-hoc; defer.
- **Animated QR scanner cliente-side** (Out of Scope ya en REQUIREMENTS) — bloqueado por HTTPS secure-context requirement de `getUserMedia`.
- **Exportar history como ZIP** (descargar todo el histórico de golpe) — útil v1.x; defer hasta que demanda lo justifique.
- **Multi-language UI (i18n)** — castellano único en v1. Estructura de strings centralizada permite añadir locales después sin rework.
- **Themes custom más allá de light/dark** — defer; light + dark + auto cubren el 99%.

### Reviewed Todos (not folded)
None — sin todos en backlog (`gsd-tools todo match-phase 2` retornó 0 matches).

</deferred>

---

*Phase: 02-spa-frontend-history*
*Context gathered: 2026-05-06*
