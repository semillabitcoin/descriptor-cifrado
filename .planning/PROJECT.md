# BED Start9 App

## What This Is

App s9pk para StartOS 0.4.0 que cifra y descifra descriptors de Bitcoin siguiendo el draft BIP "Bitcoin Encrypted Backup" (PR `bitcoin/bips#1951`, autor pythcoiner / Wizardsardine). Permite a holders multisig distribuir backups del descriptor con redundancia masiva sin sacrificar privacidad: el `.bed` cifrado solo se descifra con una xpub cosigner.

## Core Value

Un holder StartOS puede pegar un descriptor multisig y obtener un `.bed` cifrado (binario, armored o QR) sin instalar ni compilar nada, y luego recuperar ese descriptor pegando el `.bed` + cualquier xpub cosigner — todo local, sobre Tor, sin telemetría.

**Compatibilidad de formato:** los `.bed` producidos por esta app son cripto-compatibles con Liana producción (crate `bitcoin-encrypted-backup` v0.0.2, AES-256-GCM, magic `BEB`). Un `.bed` exportado desde Liana se descifra aquí y viceversa. Si Liana en el futuro bumpea a un release distinto del crate, abriremos un nuevo ciclo de migración — no perseguimos HEAD master del autor.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- [x] Validación BIP: rechazar descriptors sin derivación multipath `<a;b>/*` con `a≠b` *(Validado en Phase 1: crypto-core-http-api — substrato HTTP devuelve 422 MISSING_MULTIPATH_WILDCARD; relajado en quick task 260507-v6e para aceptar cualquier par distinto, no solo `<0;1>/*`)*
- [x] Borrado seguro del descriptor en claro de memoria tras cifrar *(Validado en Phase 1: `ZeroizingDescriptor` newtype + `Zeroizing<String>` en primera línea de handlers)*
- [x] Cifrado de descriptor: pegar descriptor → recibir `.bed` binario descargable *(Validado en Phase 2: spa-frontend-history — TabCifrar + CifrarOutputs)*
- [x] Salida armored estilo PGP (`-----BEGIN BITCOIN ENCRYPTED BACKUP-----`) *(Validado en Phase 2: copia con feedback dual toast+label)*
- [x] Salida QR (PNG descargable) generado del base64 armored *(Validado en Phase 2: descarga directa desde CifrarOutputs)*
- [x] Descifrado simétrico: subir `.bed` (binario o armored) + xpub → recibir descriptor en claro *(Validado en Phase 2: TabDescifrar + drop-zone + DescifrarOutputs + AnimatedQrModal BBQR lazy)*
- [x] Modo histórico opt-in: toggle persiste `.bed` en `/data/encrypted/<timestamp>-<short-id>.bed` *(Validado en Phase 2: 4 endpoints + HIST-03 enforced — descriptor en claro NUNCA toca disco)*
- [x] Listar y borrar entradas del historial desde la UI *(Validado en Phase 2: TabHistorial + HistoryEntryDetailModal + ConfirmDeleteModal)*
- [~] ~~Panel "modelo de amenazas" inline en la UI~~ *(Retirado 2026-05-06 sesión 8 — preferencia de UI limpia; el modelo de amenazas se documenta en README solo)*

### Active

<!-- Current scope. Building toward these. -->
- [ ] Empaquetado s9pk para StartOS 0.4.0 con Tor onion + LAN
- [ ] Imagen runtime distroless ~5–10 MB
- [ ] Documentación con modelo de amenazas explícito en README

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- Integración con app File Browser de StartOS — diferida a v2; necesita verificar APIs reales (HTTP filebrowser upstream, volúmenes compartidos, `user-files`) en StartOS 0.4.0
- Hardware wallet support (`devices` feature de la crate) — USB no llega al contenedor StartOS 0.4.0
- Cross-platform a Umbrel — solo StartOS por ahora; portar es follow-up posterior
- Multi-usuario / autenticación propia — protección delegada al Tor onion + auth de StartOS
- Persistencia del descriptor en claro — NUNCA, solo el `.bed` cifrado puede persistirse

## Context

- **Por qué existe:** En multisig clásico cada signer guarda el descriptor completo (todas las xpubs). Comprometer una sola ubicación basta para que un atacante derive todas las direcciones y vea saldo/historial — la privacidad real es 1-de-N, no M-de-N. El BIP "Bitcoin Encrypted Backup" cifra el descriptor con las propias xpubs de los cosigners; cualquier participante (1-de-N) puede descifrarlo en solitario, permitiendo distribuir backups con redundancia sin riesgo.
- **Referencias clave:**
  - BIP draft: https://github.com/bitcoin/bips/pull/1951
  - Hilo Delving Bitcoin: https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607
  - Crate Rust + CLI: https://github.com/pythcoiner/encrypted_backup
  - GUI nativa de referencia: https://github.com/pythcoiner/bed
  - Liana v13 ya implementa archivos `.bed`
- **Audiencia:** holders StartOS con multisig que quieren backup redundante del descriptor sin compilar la CLI Rust.
- **Premisa de seguridad clave:** ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig.

## Constraints

- **Tech stack**: Rust + axum + tokio — importar la crate `bitcoin-encrypted-backup` directamente (NO shellear la CLI `beb`)
- **Crate pin**: `bitcoin-encrypted-backup` tag `v0.0.2` (rev `cd7ee382bf5ca0798d4f81697e2f9efb5e32fe40`) — único release publicado, compat con Liana producción. NUNCA HEAD master (formato cripto distinto: ChaCha20-Poly1305 vs AES-256-GCM, magic `BIPXXX` vs `BEB`)
- **Tech stack**: Frontend SPA mínima vanilla JS o Svelte servida desde el mismo backend — sin CDN externo, sin telemetría, sin fonts remotas
- **Compatibilidad**: miniscript v0.12.x (la crate soporta features `miniscript_12_0` y `miniscript_12_3_5`); features de la crate = `miniscript_12_3_5` + `rand` (NO `base64`/`devices`/`cli`/`tokio`)
- **BIP**: descriptors deben usar derivación multipath `<a;b>/*` con `a≠b` (típicamente `<0;1>/*`; Liana recovery usa `<2;3>/*`); sin esta derivación, gastar desde dirección 0 expone la xpub on-chain y rompe el cifrado
- **Plataforma**: solo StartOS 0.4.0 — invocar skill `start9-packaging` cuando llegue empaquetado
- **Imagen**: build con `rust:slim`, runtime con `distroless/cc`, target ~5–10 MB
- **Acceso de red**: Tor onion + LAN, no clearnet
- **Persistencia**: descriptor en claro NUNCA persiste — solo `.bed` cifrado en `/data/encrypted/` (modo opt-in)
- **Repo**: organización `semillabitcoin` (preferencia del usuario, no PRs/forks externos)
- **GHCR**: hacer paquetes públicos tras primer push o el deploy falla
- **Git**: usar email noreply `55397917+4rkad@users.noreply.github.com`
- **Idioma**: comunicación en castellano (no argentino)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Importar `bitcoin-encrypted-backup` como crate, no shellear `beb` | Reduce superficie de ataque, evita exec, mejor manejo de errores tipados | — Pending |
| Modo histórico es opt-in (B), default ephemeral | Default seguro: app olvida tras entregar resultado; usuario decide explícitamente persistir | — Pending |
| Solo persistir `.bed` cifrado, NUNCA descriptor en claro | Modelo de amenazas: si StartOS se compromete, el `.bed` ya cifrado no expone nada nuevo | — Pending |
| Tres formatos de salida (binario, armored, QR) en v1 | Cubre los tres flujos de transporte: archivo, copy/paste, papel | — Pending |
| File Browser integration diferida a v2 | APIs reales de StartOS 0.4.0 sin verificar; download directo del navegador funciona ya | — Pending |
| Hardware wallet fuera de scope | USB no llega al contenedor StartOS 0.4.0 | — Pending |
| Solo StartOS 0.4.0 en v1 | Foco; cross-platform a Umbrel es follow-up | — Pending |
| Tor onion + LAN, sin clearnet | App maneja descriptors; clearnet aumenta superficie sin beneficio | — Pending |
| Crate `bitcoin-encrypted-backup` pinneada exact al tag `v0.0.2` (rev `cd7ee382`) | Único release publicado del crate; compat cripto con Liana producción (AES-256-GCM, magic `BEB`); HEAD master usa ChaCha20-Poly1305 + magic `BIPXXX` placeholder y rompe interop. Pin original `17b69b71` (HEAD) se sustituyó por `cd7ee382` (tag v0.0.2) en quick task `260506-sr7` tras detectar incompatibilidad con `.bed` real de Liana | ✓ Phase 1 + sesión 7 |
| Workspace lints `unwrap_used = "deny"` + `expect_used = "deny"` | Garantiza no-panic en request path; clippy `-D warnings` lo enforce | ✓ Phase 1 |
| Bind 127.0.0.1:8080 (no clearnet binding) | StartOS rutea externamente vía Tor + LAN; binding privado evita exposure accidental | ✓ Phase 1 |
| Bans cargo-deny: openssl-sys, native-tls, async-hwi | TLS lo termina StartOS; rustls everywhere para distroless | ✓ Phase 1 |
| Retirar panel UI "modelo de amenazas" (componente `ThreatModel.svelte`) | Preferencia de UI limpia; el modelo de amenazas se documenta en README (DOC-01/DOC-02) que el usuario lee al instalar — duplicarlo en cada carga de la UI añade ruido visual sin valor incremental | ✓ Sesión 8 (2026-05-06) |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-06 — Phase 2 complete + crate pivot a v0.0.2 (interop Liana confirmada por UAT)*
