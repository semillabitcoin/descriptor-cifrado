---
phase: quick/260506-sr7
plan: 01
subsystem: crypto-core
tags: [interop, liana, bed, v0.0.2, aes-gcm, fixture, smoke-test]
dependency_graph:
  requires: []
  provides: [interop-liana-bed-decrypt]
  affects: [bed-core, bed-server, tests]
tech_stack:
  added: []
  patterns: [bitcoin-encrypted-backup@v0.0.2, AES-256-GCM, magic-BEB]
key_files:
  created: []
  modified:
    - crates/core/Cargo.toml
    - crates/core/src/decrypt.rs
    - crates/server/src/routes/decrypt.rs
    - crates/core/tests/round_trip.rs
    - crates/server/tests/round_trip.rs
    - crates/core/tests/fixtures/wallet.bed
decisions:
  - "Pinned bitcoin-encrypted-backup al tag v0.0.2 (sha cd7ee382) por compatibilidad con Liana; HEAD master usa algoritmo y magic distintos e incompatibles."
  - "Feature base64 retirada del array features: no existe en v0.0.2 (añadida en commit 55054b4 posterior)."
  - "Fixture wallet.bed regenerado con la crate v0.0.2 in-tree via example regen_fixture (eliminado tras uso)."
  - "Test decrypt_with_binary_bed_works actualizado: v0.0.2 no acepta base64 suelto, ahora envía binario crudo."
metrics:
  duration: "~30 min"
  completed: "2026-05-06"
  tasks: 4
  files: 6
---

# Quick Task 260506-sr7: Migrar bitcoin-encrypted-backup a v0.0.2 — Summary

**One-liner:** Pinneado `bitcoin-encrypted-backup` al tag publicado v0.0.2 (AES-256-GCM, magic `BEB`) para restaurar interop end-to-end con Liana en producción; smoke test acid contra `test.bed` devuelve HTTP 200 con descriptor real.

## Causa Raíz

El pin anterior apuntaba a HEAD master (`17b69b71`) de `bitcoin-encrypted-backup`. Ese HEAD había roto retrocompatibilidad frente a v0.0.2 en tres dimensiones:

| Dimensión | v0.0.2 (tag publicado, Liana) | HEAD master (anterior pin) |
|-----------|-------------------------------|---------------------------|
| Magic bytes | `BEB` | `BIPXXX` (placeholder) |
| Algoritmo simétrico | AES-256-GCM | ChaCha20-Poly1305 |
| Derivación de claves | Estándar | x-only + BIP340 |

Liana en producción usa `bitcoin-encrypted-backup = "=0.0.2"`. Cualquier `.bed` generado por Liana tiene magic `BEB` y cifrado AES-256-GCM. Con el pin a HEAD, `POST /api/decrypt` fallaba en la llamada a `set_encrypted_payload` (magic mismatch + error de descifrado).

## Cambios Concretos

### Task 1 — `crates/core/Cargo.toml` (commit `8ab70e3`)
- Rev cambiado de `17b69b71...` a `cd7ee382bf5ca0798d4f81697e2f9efb5e32fe40` (tag v0.0.2).
- Feature `"base64"` retirada del array `features` (no existe en v0.0.2; fue añadida en commit posterior `55054b4`).
- Comentario reescrito explicando el motivo del pin y el peligro de migrar a HEAD.

### Task 2 — Comentarios obsoletos (commit `ed1bea7`)
- `crates/core/src/decrypt.rs`: doc actualizado — magic `BEB`, sin autodetección de base64, solo binario crudo.
- `crates/server/src/routes/decrypt.rs`: comentario del auto-detect path reescrito en castellano con referencia a v0.0.2.

### Task 3 — Fixture y tests (commit `8d9d2f0`)
- `crates/core/tests/fixtures/wallet.bed`: regenerado con la crate v0.0.2 via `cargo run --example regen_fixture` (example eliminado tras uso). Fixture nuevo: 611 bytes, magic `BEB` (0x42 0x45 0x42).
- `crates/core/tests/round_trip.rs`: assert cambiado de `b"BIPXXX"` a `b"BEB"`.
- `crates/server/tests/round_trip.rs` (Rule 1 - Bug): test `decrypt_with_binary_bed_works` corregido — la v0.0.2 no acepta base64 suelto; ahora decodifica `bed_b64` → binario y lo envía como bytes en el multipart.

## Resultado Smoke Test Acid

**Binario:** `/workspace/descriptor-cifrado/target/release/bed-server`
**Crate compilada:** `bitcoin-encrypted-backup v0.0.2` (sha `cd7ee382`)
**Comando:**
```
curl -X POST http://127.0.0.1:8080/api/decrypt \
  -F "bed=@/workspace/claude/test.bed" \
  -F "xpub=xpub6PLACEHOLDER1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

**HTTP:** `200 OK`
**Response (primeros 200 caracteres):**
```json
{"descriptor":"wsh(or_d(pk([7408e869/48'/0'/0'/2']xpub6PLACEHOLDER1xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx/<0;1>/*),and_v
```

El descriptor descifrado empieza con `wsh(or_d(...))` — descriptor real de Liana. Interop confirmada end-to-end.

## Nota sobre Fixture Legacy

El fixture `/tmp/bed-test/wallet.bed` (generado por la CLI `beb` de HEAD master) ahora es incompatible con nuestra build. Es correcto: fue producido con algoritmo distinto (ChaCha20-Poly1305, magic `BIPXXX`). Si se necesita un fixture externo compatible con v0.0.2, regenerar con `beb` v0.0.2 o usar la CLI del repositorio en el tag `v0.0.2`.

## Deviaciones del Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Corrección de test decrypt_with_binary_bed_works**
- **Encontrado durante:** Task 3 (cargo test --workspace --release)
- **Problema:** El test enviaba `bed_b64` (texto base64) directamente en el multipart esperando que la crate auto-detectase base64. Esta funcionalidad existe en HEAD master pero NO en v0.0.2.
- **Fix:** El test decodifica `bed_b64` con `base64::Engine::decode` y envía los bytes binarios resultantes. El handler del server pasa binario crudo a `set_encrypted_payload`, que es el contrato correcto de v0.0.2.
- **Archivos modificados:** `crates/server/tests/round_trip.rs`
- **Commit:** `8d9d2f0`

## Verificaciones Finales

- `cargo build --release` verde
- `cargo test --workspace --release`: todos los tests `ok` (round_trip_fixture, cross_implementation_decrypt_with_reference_bed, decrypt_with_wrong_xpub_fails, qr_too_large_returns_error, no_leak, history tests, server validation tests)
- `grep -q 'cd7ee382bf5ca0798d4f81697e2f9efb5e32fe40' crates/core/Cargo.toml`: PASS
- `! grep -n '"base64"' crates/core/Cargo.toml | grep -v '# NOTE'`: PASS
- `! grep -rn 'BIPXXX' crates/core/src/ crates/server/src/`: PASS
- `head -c 3 wallet.bed` = `BEB` (0x42 0x45 0x42): PASS
- `GET /` → HTTP 200: PASS
- `POST /api/decrypt` con `test.bed` → HTTP 200 + descriptor real: PASS

## Commits

| Task | Hash | Descripción |
|------|------|-------------|
| 1 | `8ab70e3` | Pin Cargo.toml a v0.0.2, retirar feature base64 |
| 2 | `ed1bea7` | Sanear comentarios BIPXXX → BEB en src/ |
| 3 | `8d9d2f0` | Regenerar fixture wallet.bed y actualizar tests |
| docs | *(este commit)* | SUMMARY.md |

## Known Stubs

Ninguno. Todos los cambios son funcionales y el smoke test confirma el flujo end-to-end completo.

## Self-Check: PASSED

- `/workspace/descriptor-cifrado/crates/core/Cargo.toml` — FOUND, sha cd7ee382 confirmado
- `/workspace/descriptor-cifrado/crates/core/tests/fixtures/wallet.bed` — FOUND, 611 bytes, magic BEB
- Commits `8ab70e3`, `ed1bea7`, `8d9d2f0` — presentes en git log
- Smoke test HTTP 200 — confirmado en ejecución
