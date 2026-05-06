# Requirements: BED Start9 App

**Defined:** 2026-05-05
**Core Value:** Un holder StartOS puede pegar un descriptor multisig y obtener un `.bed` cifrado (binario, armored o QR) sin instalar ni compilar nada, y luego recuperarlo pegando `.bed` + cualquier xpub cosigner — todo local, sobre Tor, sin telemetría.

## v1 Requirements

### Crypto Core

- [x] **CORE-01**: La crate `bitcoin-encrypted-backup` se importa con features `miniscript_12_3_5`, `rand`, `base64` y NO con `devices`/`cli`/`tokio`, pinneada a un commit/rev exacto
- [x] **CORE-02**: Existe round-trip determinista (encrypt → decrypt con xpub válida) cubierto por test automatizado
- [x] **CORE-03**: La capa core valida que el descriptor usa derivación `<0;1>/*` y rechaza descriptors sin esa wildcard con error tipado
- [x] **CORE-04**: El descriptor en claro se envuelve en `secrecy::SecretString` desde el punto de parse y se zeroiza tras la operación
- [x] **CORE-05**: No existe `unwrap()`/`expect()` en el path de request; un panic hook genérico evita filtrar variables locales en backtraces

### Encryption Flow

- [x] **ENC-01**: Endpoint `POST /api/encrypt` (JSON) acepta un descriptor y devuelve los tres formatos disponibles
- [x] **ENC-02**: Salida binaria `.bed` descargable desde la UI
- [x] **ENC-03**: Salida armored estilo PGP con cabeceras `-----BEGIN BITCOIN ENCRYPTED BACKUP-----` y botón "copiar al portapapeles"
- [x] **ENC-04**: Salida QR PNG descargable generada del armored base64; si excede capacidad QR ECC-L (~2,900 B) devuelve error descriptivo en vez de QR ilegible
- [x] **ENC-05**: La UI muestra errores de validación (descriptor inválido, sin `<0;1>/*`, parsing fallido) inline y específicos

### Decryption Flow

- [x] **DEC-01**: Endpoint `POST /api/decrypt` (multipart) acepta `.bed` (binario o armored pegado) + xpub (texto o archivo) y devuelve el descriptor en claro
- [x] **DEC-02**: La UI permite pegar armored o subir archivo binario indistintamente para el mismo flujo
- [x] **DEC-03**: La UI permite pegar xpub o subir archivo con xpub
- [x] **DEC-04**: El descriptor recuperado se muestra con botón "copiar al portapapeles" y nunca se persiste
- [x] **DEC-05**: El parser tolera espacios en blanco/indentación en el armored pegado

### History Mode (Opt-In)

- [x] **HIST-01**: Toggle en la UI activa modo "guardar historial"; default es ephemeral (cifra → entrega → olvida)
- [x] **HIST-02**: Con toggle activo, los `.bed` resultantes se persisten en `/data/encrypted/<timestamp>-<short-id>.bed`
- [x] **HIST-03**: El descriptor en claro NUNCA se persiste en disco (test de CI hace grep del descriptor sobre archivos guardados)
- [x] **HIST-04**: Endpoint `GET /api/history` lista entradas vía directory scan de `/data/encrypted/`
- [x] **HIST-05**: Endpoint `DELETE /api/history/:id` borra una entrada
- [ ] **HIST-06**: La UI lista y permite borrar entradas del historial

### Frontend

- [x] **UI-01**: SPA Svelte 5 + Vite 6 servida desde el binario vía `rust-embed`, sin CDN externo, sin telemetría, sin fonts remotas
- [x] **UI-02**: La UI presenta dos pestañas/secciones simétricas: "Cifrar" y "Descifrar"
- [x] **UI-03**: La UI muestra modelo de amenazas resumido visible (no solo en README)

### Security Hygiene

- [x] **SEC-01**: TraceLayer configurado con `skip_all` en handlers sensibles; test asegura que un descriptor conocido no aparece en logs capturados
- [x] **SEC-02**: Servidor binda en `127.0.0.1:8080`, no en `0.0.0.0` (StartOS gestiona el routing externo)
- [x] **SEC-03**: El proyecto usa `rustls` en todo lugar; `cargo deny` en CI rechaza dependencias con `openssl-sys` o `native-tls`

### Packaging — Docker / GHCR

- [ ] **PKG-01**: Dockerfile multi-stage `rust:slim` → `distroless/cc-debian12` produce imagen ≤25 MB
- [ ] **PKG-02**: Imagen multi-arch (amd64 + arm64) publicada en GHCR bajo organización `semillabitcoin`
- [ ] **PKG-03**: Build de CI corre `ldd` sobre el binario y falla si aparece `libssl` o cualquier lib no presente en distroless
- [ ] **PKG-04**: La imagen GHCR se marca pública inmediatamente tras el primer push

### Packaging — StartOS s9pk

- [ ] **S9-01**: Repo `semillabitcoin/bed-startos` inicializado desde `hello-world-startos` rama `update/040`
- [ ] **S9-02**: Manifest TypeScript declara interfaces (Tor + LAN auto-generadas vía `bindPort`) y volume `main` que cubre `/data/encrypted/`
- [ ] **S9-03**: Health check usa `sdk.healthCheck.checkPortListening`
- [ ] **S9-04**: La app se instala y arranca en un dispositivo StartOS 0.4.0 real (no solo `docker run`)
- [ ] **S9-05**: La actualización de la app preserva el contenido del historial (`/data/encrypted/`)

### Documentation & CI

- [ ] **DOC-01**: README documenta el modelo de amenazas explícito (lo que protege y lo que NO protege)
- [ ] **DOC-02**: README incluye el aviso clave: "ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig"
- [x] **CI-01**: Pipeline de CI corre `cargo audit` + `cargo deny` y falla en vulnerabilidades / licencias prohibidas
- [x] **CI-02**: Pipeline de CI corre el test de round-trip y el test de no-leak (descriptor no aparece en logs ni en archivos persistidos)

## v2 Requirements

### File Browser Integration

- **FB-01**: Integración con la app File Browser de StartOS para escribir `.bed` directamente al volume compartido
- **FB-02**: Decisión entre HTTP API filebrowser upstream vs volúmenes compartidos vs `user-files` host

### UX Polish

- **UX2-01**: Drag-and-drop para `.bed` y xpub
- **UX2-02**: "Test decrypt" round-trip transparente antes de mostrar éxito
- **UX2-03**: Display de checksum del descriptor recuperado para verificación visual
- **UX2-04**: Mensajes específicos por tipo de error de sintaxis del descriptor

### Cross-Platform

- **XPLAT-01**: Port a Umbrel App Store

### Persistence

- **PERS-01**: Configuración persistente del toggle "guardar historial" cruza reinicios del contenedor (file-models config)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Hardware wallet support (`devices` feature de la crate) | USB no llega al contenedor StartOS 0.4.0 |
| Multi-usuario / autenticación propia | Protección delegada al Tor onion + auth StartOS |
| Camera / scanner QR en la UI | HTTPS secure-context requirement; complica acceso por Tor/LAN |
| Shamir Secret Sharing | Modelo criptográfico distinto; el autor del BIP lo rechaza explícitamente |
| Cifrado de datos arbitrarios | Scope creep; la crate es específica de descriptors |
| Persistencia del descriptor en claro | Violaría el modelo de amenazas — NUNCA |
| SDK Umbrel en v1 | Foco en una sola plataforma; cross-platform es follow-up |
| Embedded DB (sled / SQLite) | sled abandonado, SQLite C-FFI rompe distroless; directory scan es suficiente |
| Server bind 0.0.0.0 | StartOS rutea externamente; loopback evita exposición lateral |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01 | Phase 1 | Complete |
| CORE-02 | Phase 1 | Complete |
| CORE-03 | Phase 1 | Complete |
| CORE-04 | Phase 1 | Complete |
| CORE-05 | Phase 1 | Complete |
| ENC-01 | Phase 1 | Complete |
| ENC-02 | Phase 1 | Complete |
| ENC-03 | Phase 1 | Complete |
| ENC-04 | Phase 1 | Complete |
| ENC-05 | Phase 1 | Complete |
| DEC-01 | Phase 1 | Complete |
| DEC-02 | Phase 1 | Complete |
| DEC-03 | Phase 1 | Complete |
| DEC-04 | Phase 1 | Complete |
| DEC-05 | Phase 1 | Complete |
| SEC-01 | Phase 1 | Complete |
| SEC-02 | Phase 1 | Complete |
| SEC-03 | Phase 1 | Complete |
| CI-01 | Phase 1 | Complete |
| CI-02 | Phase 1 | Complete |
| UI-01 | Phase 2 | Complete |
| UI-02 | Phase 2 | Complete |
| UI-03 | Phase 2 | Complete |
| HIST-01 | Phase 2 | Complete |
| HIST-02 | Phase 2 | Complete |
| HIST-03 | Phase 2 | Complete |
| HIST-04 | Phase 2 | Complete |
| HIST-05 | Phase 2 | Complete |
| HIST-06 | Phase 2 | Pending |
| PKG-01 | Phase 3 | Pending |
| PKG-02 | Phase 3 | Pending |
| PKG-03 | Phase 3 | Pending |
| PKG-04 | Phase 3 | Pending |
| S9-01 | Phase 4 | Pending |
| S9-02 | Phase 4 | Pending |
| S9-03 | Phase 4 | Pending |
| S9-04 | Phase 4 | Pending |
| S9-05 | Phase 4 | Pending |
| DOC-01 | Phase 4 | Pending |
| DOC-02 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 40 total (5 CORE + 5 ENC + 5 DEC + 6 HIST + 3 UI + 3 SEC + 4 PKG + 5 S9 + 2 DOC + 2 CI)
- Mapped to phases: 40/40
- Unmapped: 0

Note: The REQUIREMENTS.md header stated 36 requirements; actual count by category is 40. All 40 are mapped.

---
*Requirements defined: 2026-05-05*
*Last updated: 2026-05-05 — traceability populated by roadmapper*
