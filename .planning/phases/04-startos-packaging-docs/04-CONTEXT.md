# Phase 4: StartOS Packaging + Docs - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 envuelve la imagen multi-arch `ghcr.io/semillabitcoin/descriptor-cifrado` (entregable de Phase 3) en un manifest `.s9pk` para StartOS 0.4.0, instala en device real, y documenta el threat model en README inglés. El repo wrapper es `semillabitcoin/bed-startos` clonado de `hello-world-startos` rama `update/040` (S9-01); el manifest TypeScript declara interfaces Tor + LAN auto-generadas, volume `main` que cubre `/data/encrypted/`, health check `checkPortListening`, y publica `.s9pk` artifact en releases de GitHub + en el registry propio Semilla Bitcoin tras validación en device real.

**Fuera de Phase 4:**
- i18n EN+ES de la UI — nueva fase (Phase 5 nueva en milestone v0.0.2; añadir post-cierre Phase 4 vía `/gsd:add-phase`).
- Cualquier cambio al backend Rust o frontend Svelte (Phase 1+2 cerradas; Phase 3 entrega imagen).
- Publicación en StartOS registry oficial (review Start9 indefinido — defer).
- SBOM / cosign signing del `.s9pk` — defer.
- Migration scripts automáticos — no aplicable v1 (ver D-06).
- Sanitización del historial git de `descriptor-cifrado` (sesión separada con `git-filter-repo`) — pre-requisito independiente para flipar repo a público y validar PKG-04 sin auth; no bloquea Phase 4 (s9pk consume imagen privada con `docker login` en build host si el registry lo requiere).

</domain>

<decisions>
## Implementation Decisions

### Image Pin Strategy
- **D-01:** El manifest `bed-startos` referencia la imagen GHCR por `@sha256:digest` (no `:latest`, no tag semver móvil). Razón: el `.s9pk` debe ser determinista — el mismo artifact siempre instala el bit-exacto build verificado. Bump del digest es 1 línea cuando se publica nuevo backend (acoplado a versioning, ver D-05). El digest se obtiene de `docker buildx imagetools inspect ghcr.io/semillabitcoin/descriptor-cifrado:vX.Y.Z` tras el push del tag git.

### Documentation (DOC-01 + DOC-02)
- **D-02:** README está en **inglés** (audiencia GitHub público + registry); términos castellanos preservados solo donde citan UI labels actuales (la UI sigue en castellano hasta Phase 5 i18n). Estructura en 6 secciones:
  1. **Quickstart / TL;DR** (≤5 líneas) — qué hace, cómo instalar, regla de oro DOC-02 ("never co-locate `.bed` and a cosigner xpub")
  2. **Usage** — flujo cifrar + flujo descifrar con screenshots de las 2 tabs
  3. **Threat model** — 3 sub-secciones: qué protege / qué NO protege / supuestos del modelo
  4. **Crypto details** — AES-256-GCM, magic `BEB`, BIP draft PR `bitcoin/bips#1951`, interop Liana v13+ (crate v0.0.2)
  5. **Common pitfalls** — descriptor sin `<0;1>/*`, xpub vs descriptor-style con `[fingerprint/path]` prefix, QR ECC-L size limit (~2,900 B), modo histórico opt-in
  6. **References** — BIP PR, encrypted_backup crate (pythcoiner), Delving Bitcoin thread, Liana docs
- **D-03:** DOC-02 aparece **dos veces** en el README: en TL;DR (visible al primer scroll) y en Threat Model §"What it does NOT protect against". Redundancia intencional — es el aviso más crítico del proyecto.

### App Identity (StartOS dashboard)
- **D-04:** Nombre amigable: **"BED — Bitcoin Encrypted Backup"**.
  - `manifest.title = "BED"`
  - `manifest.description.short = "Encrypt Bitcoin descriptor backups for redundant multisig storage"`
  - `manifest.description.long` describe el threat model en 1-2 párrafos con link al README extendido.
- **D-05:** Icono **v1 = logotipo "BED" textual** (opción C de discusión). Generar SVG en `plan-phase`:
  - Texto "BED" en monospace bold (JetBrains Mono o equivalente, alineado con el font self-hosted del frontend Phase 2)
  - Paleta Semilla Bitcoin: fondo negro `#0c0c0c` (o naranja Bitcoin `#f7931a`), texto contrastante
  - Cuadrado, exportar PNG 1024×1024 (StartOS spec mínimo)
  - Si el preview no convence al usuario, follow-up con diseñador para opción D (sobre cifrado con sello Bitcoin) o B (3 llaves entrelazadas) — defer indefinido.

### Registry Deployment
- **D-06:** Distribución en 2 canales:
  1. **Sideload (siempre)**: `.s9pk` artifact publicado en GitHub Releases de `semillabitcoin/bed-startos` con cada tag `vX.Y.Z`. Vía universal — cualquier holder StartOS puede sideload sin depender del registry.
  2. **Registry propio Semilla Bitcoin** (locked tras S9-04): publicar al registry propio del usuario (`project_startos_registry.md`) tras validación en device real. Catálogo controlado, audiencia primaria castellana + bilingüe.
- **D-07:** StartOS registry oficial Start9 → **defer milestone futuro**. Requiere review Start9, timeline indefinido. v1 prioriza shippeable, no inclusión en catálogo oficial.

### Versioning Strategy
- **D-08:** Versionado **acoplado** entre `descriptor-cifrado` y `bed-startos`:
  ```
  descriptor-cifrado v0.1.0   →  GHCR :v0.1.0 + sha256:XXX
  bed-startos v0.1.0          →  manifest pinea sha256:XXX, label v0.1.0
  ```
  Ambos repos comparten semver. Mensaje al usuario es "BED v0.1.0" (un solo número). El bump manual es: tag git en `descriptor-cifrado` → CI publica imagen → leer digest → bump manifest `bed-startos` → tag git → release `.s9pk`.
- **D-09:** Primer release **v0.1.0** (no v0.0.1). Razón: el milestone actual es `v0.0.2` interno del proyecto, pero el primer release público del producto BED es lo suficientemente completo (cripto core + UI + s9pk + docs) para semver minor estable.

### Update Path Doctrine
- **D-10:** **Zero migration code en v1**. La app no almacena la xpub del usuario, por tanto no puede re-cifrar `.bed` viejos automáticamente. Doctrina:
  - **Patches/minor (v0.1.x)**: bump backend cosmético/UI/bugfix → `.bed` existentes siguen leyéndose. S9-05 garantiza preservación de `/data/encrypted/`.
  - **Breaking en formato `.bed`** (ej. crate bumpea HEAD ChaCha20 vs v0.0.2 AES): NO migration auto — milestone nuevo, archivamos versión vieja en CHANGELOG, doctrina "para descifrar `.bed` viejos: usar BED v0.x archivado". Locked en PROJECT.md "Compatibilidad de formato".
  - **Schema `/data/encrypted/`** (filename format `<YYYYMMDDTHHMMSSZ>-<8hex>.bed` Phase 2): si rompe en futuro, script idempotente al startup. v1 NO contempla.
- **D-11:** El formato `.bed` es **contrato externo** (interop Liana producción crate v0.0.2). No se rompe sin milestone explícito. Documentado en README §Crypto details.

### Repo Strategy
- **D-12:** Repo separado `semillabitcoin/bed-startos`, clonado de `Start9Labs/hello-world-startos` rama `update/040` (S9-01 locked en REQUIREMENTS). Inicialmente **PRIVATE** (consistencia con `descriptor-cifrado` que sigue privado por historial sucio). Flip a público cuando ambos estén sanitizados.
- **D-13:** Author de commits: noreply `55397917+4rkad@users.noreply.github.com` (feedback `feedback_git_noreply_email.md`). Aplica al primer commit de inicialización + clones del template.

### Networking & Volumes
- **D-14:** Interfaces declaradas en `manifest.ts` vía StartOS SDK 0.4.0 `bindPort`:
  - **Tor onion** auto-generada (`type: 'http'`, internal port 8080)
  - **LAN** `.local` auto-generada (`type: 'ui'`, `ssl: false` — StartOS termina TLS en la capa LAN)
  - Sin clearnet, sin toggle entre interfaces (ambas activas siempre).
- **D-15:** Volume `main` cubre `/data/encrypted/` (HIST-02 path). Ownership UID 65532 (distroless nonroot, Phase 3 D-13). El manifest debe asegurar que StartOS crea el volume con ownership correcto antes del primer arranque (StartOS SDK lo gestiona si declaramos `volumes.main` con `mountpoint: "/data/encrypted"`).

### Health Check
- **D-16:** `sdk.healthCheck.checkPortListening` apuntando a `127.0.0.1:8080` (S9-03 locked). No HTTP `/api/health` endpoint custom en v1 — `checkPortListening` es suficiente porque el binario solo bindea el puerto cuando axum está listo. Si en el futuro queremos liveness más fino (verificar que `bitcoin-encrypted-backup` carga correctamente, que `BED_DATA_DIR` es escribible), añadimos endpoint `/api/health` en milestone posterior.

### Build & Release Pipeline
- **D-17:** El repo `bed-startos` tiene su propio CI (GitHub Actions) que:
  1. Corre `start-sdk pack` para producir `.s9pk`
  2. Sube `.s9pk` como release asset en tag push
  3. (Opcional v1) corre `start-sdk verify` o equivalente smoke-test si SDK 0.4.0 lo expone
- **D-18:** El test final S9-04 (install en device StartOS 0.4.0 real) es **manual y bloqueante**. Sin device físico verificado, Phase 4 no se cierra. UAT pasa por: install, abrir Tor URL, cifrar descriptor real, descifrar, comprobar persistencia tras reboot del contenedor (S9-05).

### Claude's Discretion
- Estructura de directorios dentro de `bed-startos` (typical: `manifest.ts` + `instructions.md` + `LICENSE` + `icon.png` + `assets/`). Planner decide tras invocar skill `start9-packaging`.
- Versión exacta de Start9 SDK npm package (`@start9labs/start-sdk` 0.4.x latest beta). Researcher verifica al momento del plan.
- Si añadir `actions.ts` para acciones custom (ej. "Reset history") o dejar v1 sin custom actions. Recomendación: sin actions v1, `BED_DATA_DIR` se gestiona desde la UI del propio backend.
- Estructura del manifest properties (env vars expuestas, config opcional). Recomendación: zero env vars configurables en v1 — defaults del binario (`BED_DATA_DIR=/data/encrypted`, `127.0.0.1:8080`) son suficientes.
- Nivel de detalle de los screenshots del README (real device vs mockup vs sin screenshots). Recomendación: screenshots reales tras S9-04.

### Folded Todos
None — `gsd-tools todo match-phase 4` retornó 0 matches.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project planning (este repo `descriptor-cifrado`)
- `.planning/PROJECT.md` — Constraints: solo StartOS 0.4.0, Tor + LAN, organización `semillabitcoin`, email noreply, idioma castellano (excepto README + UI futura i18n)
- `.planning/REQUIREMENTS.md` §"Packaging — StartOS s9pk" — S9-01..05 + §"Documentation & CI" — DOC-01, DOC-02
- `.planning/ROADMAP.md` §"Phase 4: StartOS Packaging + Docs" — Goal + 2 success criteria + flag `needs_research: true`
- `.planning/STATE.md` — Position actual (Phase 4 ready)
- `.planning/phases/01-crypto-core-http-api/01-CONTEXT.md` — Bind `127.0.0.1:8080` (D-16 health check), descriptor en claro nunca persiste (DOC threat model)
- `.planning/phases/02-spa-frontend-history/02-CONTEXT.md` — `BED_DATA_DIR=/data/encrypted/` default, filename format `<YYYYMMDDTHHMMSSZ>-<8hex>.bed`, modo histórico opt-in (D-15 volume + threat model)
- `.planning/phases/03-docker-ghcr/03-CONTEXT.md` — Imagen GHCR `ghcr.io/semillabitcoin/descriptor-cifrado` con UID 65532 nonroot, multi-arch amd64+arm64, digest pinning como mejor práctica (D-01, D-15)
- `IDEA.md` — Brief original

### Repos plantilla y ecosistema StartOS
- `Start9Labs/hello-world-startos` rama `update/040` — plantilla source para `bed-startos` (S9-01)
- `Start9Labs/start-os` v0.4.0-beta release notes — capabilities del SDK
- `Start9Labs/start-sdk` (TypeScript) — API actual de manifest.ts, healthCheck, bindPort, volumes
- StartOS docs portal — https://docs.start9.com/0.4.0/ (verificar URL actual al research)
- Skill `start9-packaging` — **invocar OBLIGATORIAMENTE en research-phase y plan-phase** (constraint del proyecto + memoria `reference_skill_start9_packaging.md`)

### External specs (web — usar `WebFetch` para verificar versiones / patrones actuales)
- BIP draft "Bitcoin Encrypted Backup" — https://github.com/bitcoin/bips/pull/1951 (referenciado en README §Crypto details)
- Delving Bitcoin thread — https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607
- Crate `bitcoin-encrypted-backup` v0.0.2 — https://github.com/pythcoiner/encrypted_backup (tag v0.0.2, rev `cd7ee382`)
- Liana docs sobre `.bed` — https://wizardsardine.com/liana/ (referenciado en README)
- StartOS SDK 0.4.0 reference — depende del estado del SDK al momento del research

### Memoria del usuario aplicable
- `project_startos_registry.md` — registry propio Semilla Bitcoin (D-06 destino primario de publicación)
- `reference_skill_start9_packaging.md` — invocar skill al empaquetar s9pk StartOS 0.4.0
- `feedback_test_before_push.md` — tests locales no detectan problemas reales; S9-04 (test en device real) es bloqueante
- `feedback_audit_git_history_pre_public_push.md` — auditar historial git antes de flipar repo a público (`bed-startos` también; D-12)
- `feedback_git_noreply_email.md` — `55397917+4rkad@users.noreply.github.com` en commits (D-13)
- `feedback_castellano_no_argentino.md` — strings UI en castellano peninsular; README en inglés salvo cuando cite labels UI
- `feedback_verificar_no_inventar.md` — versiones SDK + API endpoints verificados al research, no asumidos
- `reference_skill_start9_packaging.md` — workflow de empaquetado (canonical)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Imagen GHCR `ghcr.io/semillabitcoin/descriptor-cifrado` multi-arch (Phase 3)** — ya pusheada con manifest list `sha256:da3c9a1d...` (puede ser otro digest tras nuevo build, validar al plan). El `.s9pk` la consume directamente.
- **Binario `bed-server` distroless nonroot UID 65532, bind `127.0.0.1:8080`** — interfaces y health check del manifest se construyen sobre estos contratos.
- **Volume target `/data/encrypted/` con `BED_DATA_DIR` env var** — Phase 2 `02-CONTEXT.md` D-32. Manifest declara volume `main` que monta ahí.
- **Frontend SPA Svelte 5 servida desde el binario** — la UI del s9pk es `http://<onion>:8080/` o `http://bed.local:8080/`. No requiere mounts adicionales.
- **`feedback_audit_git_history_pre_public_push.md`** — antes de flipar `bed-startos` a público, auditar historial del fork de `hello-world-startos` por leaks accidentales.

### Established Patterns
- **Versión pinning de Actions** (Phase 1+3) — `bed-startos` GHA usa `@v4`/`@v5`/`@v6` canonical stable.
- **Workspace `--locked`** — no aplica directamente (`bed-startos` no es Rust workspace), pero el patrón equivalente es `npm ci` para deps del manifest TS y `start-sdk pack` reproducible.
- **OCI labels y digest pinning** (Phase 3 D-07) — el manifest `bed-startos` consume el digest de la imagen, no el tag móvil.

### Integration Points
- **GHCR pull**: el host de build de `bed-startos` debe poder pullear la imagen. Si `descriptor-cifrado` GHCR sigue privado al momento de Phase 4, el CI de `bed-startos` necesita `docker login ghcr.io` con `GITHUB_TOKEN` que tenga `read:packages`. Workaround: usar el mismo token de la org `semillabitcoin` que ya tiene acceso.
- **StartOS SDK manifest** — `bindPort(8080, { type: 'http' })` para Tor + `bindPort(8080, { type: 'ui', ssl: false })` para LAN. La API exacta se verifica al research vía skill `start9-packaging`.
- **Health check** — `sdk.healthCheck.checkPortListening({ port: 8080 })` (S9-03 directo a la API SDK).
- **Volume mount** — `volumes.main = { type: "data", mountpoint: "/data/encrypted" }` (sintaxis verificable al research).
- **Repo `bed-startos` GH Actions** — separadas del `descriptor-cifrado` workflows. CI corre `start-sdk pack`, sube `.s9pk` como release asset, opcionalmente publica al registry Semilla Bitcoin (script auxiliar).

</code_context>

<specifics>
## Specific Ideas

- **README en inglés** sirve audiencia GitHub público + registry futuro internacional. Términos castellanos preservados solo donde citen labels UI actuales (la UI sigue castellano hasta Phase 5 i18n). Cuando Phase 5 cierre, README añadirá nota sobre toggle de idioma.
- **Logotipo "BED" SVG** generado en el plan-phase con preview al usuario antes del commit. Si el preview no convence, fallback es opción D (sobre cifrado con sello Bitcoin) requiriendo diseñador externo.
- **DOC-02 redundancia (regla de oro)**: aparece en TL;DR Y en Threat Model §"What it does NOT protect against". El usuario probablemente lee solo la primera parte del README — la regla de oro debe verla sí o sí.
- **Test S9-04 en device real es UAT bloqueante**: sin install verificado en hardware StartOS 0.4.0, Phase 4 no cierra. El plan debe incluir checkpoint de UAT explícito (`HUMAN-UAT.md` con resultados del install + cifrar real + descifrar real + reboot).
- **Test S9-05 (preservación del historial)**: tras update del s9pk (versión vieja → nueva), los `.bed` existentes en `/data/encrypted/` deben seguir listándose y descifrándose. UAT incluye este check explícito.
- **Sideload primario**: el `.s9pk` en releases de `bed-startos` es la vía garantizada. El registry Semilla Bitcoin es complementario (no bloqueante para v1).
- **Repo `bed-startos` PRIVATE inicialmente** por simetría con `descriptor-cifrado`. Flip a público cuando ambos historiales estén sanitizados. Nota: el fork de `hello-world-startos` puede tener su propio historial limpio, así que la sanitización podría limitarse a `descriptor-cifrado`.
- **Versionado primer release v0.1.0** (no v0.0.1). Producto suficientemente completo (cripto + UI + s9pk + docs) para semver minor inicial.

</specifics>

<deferred>
## Deferred Ideas

- **Phase 5: i18n EN+ES** (en este milestone v0.0.2) — añadir UI bilingüe encima de Phase 2. Ejecutar `/gsd:add-phase` tras cierre de Phase 4. Capabilities: store de locale Svelte, archivos JSON con strings, persistencia de preferencia, fallback EN, toggle visible en la UI. Dependencia: Phase 2 (UI base). Independiente de Phase 4 (s9pk).
- **StartOS registry oficial Start9** — review Start9 indefinido. Defer a milestone futuro tras feedback del registry propio Semilla Bitcoin.
- **SBOM generation** del `.s9pk` — útil compliance, no requerido v1.
- **Cosign signing del `.s9pk`** — firma criptográfica del artifact. Defer (key management).
- **Migration scripts auto** entre formatos `.bed` distintos — no aplica v1, doctrina "no auto migration" (D-10).
- **Endpoint `/api/health` custom** del backend — `checkPortListening` es suficiente v1. Si necesitamos liveness más fino (escribir prueba a `BED_DATA_DIR`, verificar carga de la crate), milestone futuro.
- **Custom `actions.ts` en manifest** (ej. "Reset history", "Export all .bed") — defer; gestión actual desde la UI propia.
- **Iconos artísticos custom** (opciones B/D del discusión) — si logotipo C v1 no convence, follow-up con diseñador externo.
- **README screenshots reales tras S9-04** — el primer release puede tener placeholder; bump posterior con screenshots de device real.
- **Multi-platform Umbrel** (XPLAT-01 de REQUIREMENTS v2) — defer milestone futuro.
- **FB-01/FB-02 File Browser integration** — defer v2 (locked en PROJECT.md Out of Scope).

### Reviewed Todos (not folded)
None — `gsd-tools todo match-phase 4` retornó 0 matches.

</deferred>

---

*Phase: 04-startos-packaging-docs*
*Context gathered: 2026-05-07*
