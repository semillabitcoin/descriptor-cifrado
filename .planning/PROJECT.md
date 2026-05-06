# BED Start9 App

## What This Is

App s9pk para StartOS 0.4.0 que cifra y descifra descriptors de Bitcoin siguiendo el draft BIP "Bitcoin Encrypted Backup" (PR `bitcoin/bips#1951`, autor pythcoiner / Wizardsardine). Permite a holders multisig distribuir backups del descriptor con redundancia masiva sin sacrificar privacidad: el `.bed` cifrado solo se descifra con una xpub cosigner.

## Core Value

Un holder StartOS puede pegar un descriptor multisig y obtener un `.bed` cifrado (binario, armored o QR) sin instalar ni compilar nada, y luego recuperar ese descriptor pegando el `.bed` + cualquier xpub cosigner — todo local, sobre Tor, sin telemetría.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- [x] Validación BIP: rechazar descriptors sin derivación `<0;1>/*` *(Validado en Phase 1: crypto-core-http-api — substrato HTTP devuelve 422 MISSING_MULTIPATH_WILDCARD)*
- [x] Borrado seguro del descriptor en claro de memoria tras cifrar *(Validado en Phase 1: `ZeroizingDescriptor` newtype + `Zeroizing<String>` en primera línea de handlers)*

### Active

<!-- Current scope. Building toward these. -->

- [ ] Cifrado de descriptor: pegar descriptor → recibir `.bed` binario descargable *(API substrate listo en Phase 1; UI llega en Phase 2)*
- [ ] Salida armored estilo PGP (`-----BEGIN BITCOIN ENCRYPTED BACKUP-----`) *(API substrate listo en Phase 1)*
- [ ] Salida QR (PNG descargable) generado del base64 armored *(API substrate listo en Phase 1)*
- [ ] Descifrado simétrico: subir `.bed` (binario o armored) + xpub → recibir descriptor en claro *(API substrate listo en Phase 1)*
- [ ] Modo histórico opt-in: toggle persiste `.bed` en `/data/encrypted/<timestamp>-<short-id>.bed`
- [ ] Listar y borrar entradas del historial desde la UI
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
- **Tech stack**: Frontend SPA mínima vanilla JS o Svelte servida desde el mismo backend — sin CDN externo, sin telemetría, sin fonts remotas
- **Compatibilidad**: miniscript v0.12.x (la crate soporta features `miniscript_12_0` y `miniscript_12_3_5`)
- **BIP**: descriptors deben usar derivación `<0;1>/*`; sin esto, gastar desde dirección 0 expone la xpub on-chain y rompe el cifrado
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
| Crate `bitcoin-encrypted-backup` pinneada exact en rev `17b69b71` | No está en crates.io; pin exacto previene breaking changes silenciosos | ✓ Phase 1 |
| Workspace lints `unwrap_used = "deny"` + `expect_used = "deny"` | Garantiza no-panic en request path; clippy `-D warnings` lo enforce | ✓ Phase 1 |
| Bind 127.0.0.1:8080 (no clearnet binding) | StartOS rutea externamente vía Tor + LAN; binding privado evita exposure accidental | ✓ Phase 1 |
| Bans cargo-deny: openssl-sys, native-tls, async-hwi | TLS lo termina StartOS; rustls everywhere para distroless | ✓ Phase 1 |

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
*Last updated: 2026-05-06 — Phase 1 complete (crypto core + HTTP API substrate)*
