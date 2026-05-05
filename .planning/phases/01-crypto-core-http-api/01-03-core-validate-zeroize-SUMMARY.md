---
phase: 01-crypto-core-http-api
plan: 03
subsystem: core
tags: [rust, validation, zeroize, security, bip, multipath, property-tests]
dependency_graph:
  requires: [01-01-workspace-skeleton]
  provides: [CoreError, ZeroizingDescriptor, require_multipath_0_1]
  affects: [01-04-core-armored-qr-encrypt, crates/server (error mapping)]
tech_stack:
  added:
    - zeroize 1.8.2 (ZeroizingDescriptor newtype, already in workspace deps)
    - thiserror 2.0.18 (CoreError enum, already in workspace deps)
  patterns:
    - Newtype pattern sin Clone/Display/Debug para prevenir logging accidental (D-11)
    - for_each_key + DerivPaths.paths() para validación BIP multipath
    - #[allow(clippy::panic)] en helpers de test para satisfacer workspace lint
key_files:
  created:
    - crates/core/src/error.rs
    - crates/core/src/zeroize.rs
    - crates/core/src/validate.rs
    - crates/core/tests/validate.rs
    - crates/core/tests/fixtures/desc.txt
  modified:
    - crates/core/src/lib.rs (añadidos pub mod error/validate/zeroize + re-exports)
decisions:
  - "Usado paths() accessor directo (no fallback Display) — DerivPaths::paths() existe en miniscript 12.3.6"
  - "Xpubs de test tomados del fixture real de /tmp/bed-test/desc.txt (no BIP test vectors genéricos que no parsean)"
  - "#[allow(clippy::panic)] en parse() helper de tests — workspace lint clippy::panic deny se aplica a test targets; pattern unwrap_or_else+panic es idiomático en tests"
metrics:
  duration_minutes: 8
  tasks_completed: 2
  tasks_total: 2
  files_created: 5
  files_modified: 1
  completed_date: "2026-05-05"
---

# Phase 01 Plan 03: Core Validate + Zeroize Summary

**One-liner:** CoreError enum con mensajes en castellano, ZeroizingDescriptor newtype sin Clone/Debug/Display, y require_multipath_0_1 validando BIP <0;1>/* con 6 property tests (5 inválidos + 1 fixture real).

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Crear CoreError + ZeroizingDescriptor newtype | c63cc18 | error.rs, zeroize.rs, validate.rs (stub), lib.rs |
| 2 | Implementar require_multipath_0_1 + property tests | 5f46ee2 | validate.rs (real), tests/validate.rs, tests/fixtures/desc.txt |

## Verification Results

- `cargo build -p bed-core` — exits 0
- `cargo test -p bed-core --test validate` — exits 0, 6/6 tests pass
- `cargo clippy -p bed-core --lib -- -D warnings` — exits 0
- `cargo clippy -p bed-core --test validate -- -D warnings` — exits 0
- `ZeroizingDescriptor` sin Clone/Display/Debug — verificado con grep
- Mensaje MissingMultipathWildcard en castellano EXACTO

## Test Results

```
running 6 tests
test rejects_bare_xpub ... ok
test accepts_synthetic_2_of_3_multipath ... ok
test rejects_single_wildcard ... ok
test rejects_mixed_one_good_one_bad ... ok
test accepts_valid_fixture ... ok
test rejects_wrong_multipath_indices ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Accessor API Resolution

Se usó `mx.derivation_paths.paths()` (el accessor primario de RESEARCH.md §1.1 — Open Question).
`DerivPaths::paths()` existe en miniscript 12.3.6 (línea 87 en descriptor/key.rs: `pub fn paths(&self) -> &Vec<bip32::DerivationPath>`).
**No fue necesario el fallback Display-based.**

## Fixture SHA256

```
c4d27f271092c3ce2ac9dbd15a6e56e0a5460cf62871112eab2968ab8466a118  crates/core/tests/fixtures/desc.txt
c4d27f271092c3ce2ac9dbd15a6e56e0a5460cf62871112eab2968ab8466a118  /tmp/bed-test/desc.txt
```
Idénticos. `diff` exits 0.

## Módulos Creados

| Módulo | Archivo | Exports |
|--------|---------|---------|
| error | crates/core/src/error.rs | `CoreError` (MissingMultipathWildcard, DescriptorParse, XpubMismatch, QrTooLarge, Armored, Crypto) |
| zeroize | crates/core/src/zeroize.rs | `ZeroizingDescriptor` (new, as_str, zeroize_now) |
| validate | crates/core/src/validate.rs | `require_multipath_0_1` |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Clippy panic lint en test helper**
- **Found during:** Task 2 (cargo clippy --all-targets -D warnings)
- **Issue:** Workspace lint `clippy::panic = "deny"` se aplica también a integration test targets; el `parse()` helper del plan usa `unwrap_or_else(|e| panic!(...))` que activa el lint
- **Fix:** Añadido `#[allow(clippy::panic)]` en la función `parse()` del archivo `tests/validate.rs`
- **Files modified:** crates/core/tests/validate.rs
- **Commit:** 5f46ee2

**2. [Rule 2 - Missing context] Xpubs de test del fixture real**
- **Found during:** Task 2 (análisis del plan)
- **Issue:** El plan sugería xpubs del BIP test vector ("XPUB_A", "XPUB_B") que podrían no parsear; el plan mismo advertía de esto
- **Fix:** Usados xpubs del fixture real `/tmp/bed-test/desc.txt` (las mismas claves de la fixture válida), que sabemos que parsean correctamente
- **Files modified:** crates/core/tests/validate.rs
- **Commit:** 5f46ee2

**3. Coordinación paralela con agente 01-04**
- El agente sibling (01-04-core-armored-qr-encrypt) modificó `lib.rs` añadiendo módulos armored/encrypt/decrypt/qr mientras este plan ejecutaba
- El estado final de lib.rs tiene TODOS los módulos de ambos planes correctamente declarados
- No se requirió intervención manual — las ediciones no solapaban (módulos distintos)

## Known Stubs

Ninguno — todos los módulos de este plan están completamente implementados.

## Self-Check: PASSED

All files verified present. All commits verified in git log.

- FOUND: crates/core/src/error.rs
- FOUND: crates/core/src/zeroize.rs
- FOUND: crates/core/src/validate.rs
- FOUND: crates/core/tests/validate.rs
- FOUND: crates/core/tests/fixtures/desc.txt
- FOUND commit: c63cc18 (Task 1)
- FOUND commit: 5f46ee2 (Task 2)
