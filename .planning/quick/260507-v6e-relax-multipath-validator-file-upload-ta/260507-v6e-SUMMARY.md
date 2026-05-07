---
quick_id: 260507-v6e
phase: quick
plan: 260507-v6e
subsystem: crypto-core + spa-frontend
tags: [validator, multipath, liana, file-upload, ux]
completed: "2026-05-07"
duration_min: 45
tasks_completed: 2
files_modified: 12
decisions:
  - "XPUB_A placeholder reemplazado por xpubs reales (xpub6Euvf9GF...) en todos los fixtures"
  - "desc.txt actualizado para coincidir con wallet.bed: 3-key multisig con xpubs reales"
  - "Mensaje de error actualizado: <a;b>/* con a≠b en lugar de <0;1>/* estricto"
---

# Quick Task 260507-v6e: Relajar validador multipath + file upload + botones Limpiar

## Resumen

Validador multipath relajado de `<0;1>/*` estricto a cualquier par distinto `<a;b>/*` (con `a≠b`). Descriptors Liana recovery con `<2;3>/*` ahora devuelven 200. UI mejorada con dropzone en TabCifrar y botones Limpiar en ambos tabs.

## Commits

| Task | Hash | Mensaje |
|------|------|---------|
| Task 1 (Rust) | b804e6d | feat(crypto): relajar validador multipath a cualquier par distinto <a;b>/* |
| Task 2 (Frontend) | 53af96d | feat(ui): dropzone descriptor en TabCifrar + botón Limpiar en ambos tabs |

## Cambios por tarea

### Task 1: Validador Rust

**Archivos modificados:**
- `crates/core/src/validate.rs` — función renombrada a `require_multipath_wildcard`; condición `paths[0]=="0" && paths[1]=="1"` reemplazada por `paths[0] != paths[1]`
- `crates/core/src/encrypt.rs` — import y callsite actualizados; doc-comment actualizado
- `crates/core/tests/validate.rs` — 7 tests con xpubs reales (XPUB_B, XPUB_C, XPUB_D); `rejects_wrong_multipath_indices` → `accepts_alternate_multipath_indices`; `rejects_mixed_one_good_one_bad` → `accepts_mixed_multipath_indices`; nuevo `rejects_duplicate_multipath_indices` con `<5;5>/*`
- `crates/server/src/error.rs` — mensaje de error actualizado a `<a;b>/* con a≠b`

**Fixtures corregidos (bug preexistente):**
- `crates/core/tests/fixtures/desc.txt` — `xpub6PLACEHOLDER2xxx` → `xpub6Euvf9GFqnGhkLLeonsGfmpNSdz2oBwPMzn8tW8FJKxtfvrQFJrbQ1vp7iP8rbK9GXAG7RQK5D4dHvFRjqyawXnYPairzBM6Pnqqd2TTVMZ` (xpub real que coincide con wallet.bed)
- `crates/core/tests/fixtures/xpub.txt` — placeholder → XPUB_B real `[f91be7a4/...]xpub6DhWab.../<0;1>/*`
- `crates/server/tests/fixtures/desc.txt` — misma corrección
- `crates/server/tests/fixtures/xpub.txt` — misma corrección

### Task 2: Frontend

**Archivos modificados:**
- `frontend/src/components/TabCifrar.svelte` — dropzone drag-and-drop para cargar descriptor, botón "Limpiar", texto de ayuda con `<a;b>/*` y ejemplo `<2;3>/*`
- `frontend/src/components/TabDescifrar.svelte` — botón "Limpiar todo" siempre visible, elimina "Limpiar resultado" condicional
- `CLAUDE.md` — constraint BIP actualizado a `<a;b>/*` con `a≠b`
- `.planning/PROJECT.md` — constraint BIP y validación BIP en Requirements actualizados

## Tests

### Cargo test (validate)

```
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Cargo test (bed-core completo)

```
validate: 7 passed
round_trip: 4 passed (incluyendo cross_implementation_decrypt_with_reference_bed)
armored: 8 passed
```

### Cargo test (bed-server)

```
no_leak: 1 passed
round_trip: 2 passed
validation: 4 passed
```

### npm run build

```
✓ built in 576ms
dist/assets/index-CQLxieSv.js  73.07 kB │ gzip: 26.09 kB
```

## Smoke test

Servidor iniciado en: `BED_DATA_DIR=/tmp/bed-uat-data-v6e RUST_LOG=info .../bed-server`

| # | Descriptor | HTTP | Resultado |
|---|-----------|------|-----------|
| 3 | GET / | 200 | PASS |
| 4a | `wsh(pk(xpub.../<0;1>/*))` | 200 | PASS |
| 4b | `wsh(sortedmulti(2,xpub.../<0;1>/*,xpub.../<2;3>/*))` (regresión) | 200 | PASS — era 422 antes |
| 4c | `wsh(pk(xpub.../<5;5>/*))` | 422 MISSING_MULTIPATH_WILDCARD | PASS |

**Smoke test: 4/4 PASS**

## Comportamiento nuevo vs antiguo

| Descriptor | Antes | Ahora |
|-----------|-------|-------|
| `<0;1>/*` | 200 | 200 (sin cambio) |
| `<2;3>/*` (Liana recovery) | 422 | 200 |
| Mix `<0;1>/*` + `<2;3>/*` | 422 | 200 |
| `<5;5>/*` (par degenerado) | — (no testado) | 422 MISSING_MULTIPATH_WILDCARD |
| Bare xpub | 422 | 422 (sin cambio) |
| Single wildcard `/0/*` | 422 | 422 (sin cambio) |

## Desviaciones del plan

### Auto-fixed (Rule 1 — Bug preexistente)

**1. Fixtures con placeholder xpub inválido**
- **Encontrado durante:** Task 1 — primer `cargo test --test validate` reveló que TODOS los tests fallaban
- **Causa:** `XPUB_A = "xpub6PLACEHOLDER2xxx..."` no es un xpub válido; miniscript rechazaba el parse
- **Fix:** `XPUB_A` reemplazado por `xpub6Euvf9GF...` (el xpub real contenido en `wallet.bed`) y `XPUB_C`/`XPUB_D` para tests 3-key
- **Tests server también corregidos:** `crates/server/tests/fixtures/{desc,xpub}.txt` tenían el mismo placeholder
- **Archivos modificados:** 4 fixtures + cambio de constantes en validate.rs
- **Resultado:** cross_implementation_decrypt_with_reference_bed también pasa ahora

**2. Mensaje de error inconsistente**
- **Encontrado durante:** Task 1 — error.rs seguía diciendo `<0;1>/*` tras relajar el validador
- **Fix:** Mensaje actualizado para reflejar la nueva semántica `<a;b>/* con a≠b`
- **Verificación:** El test `validation.rs` que chequea `msg.contains("<0;1>/*")` sigue pasando (el nuevo mensaje incluye `<0;1>/*` como ejemplo)

## Estado final

- Cero referencias a `require_multipath_0_1` en el codebase
- `cargo test -p bed-core --test validate` → ok. 7 passed
- `cargo build --release` → Finished en 27s
- `npm run build` → ok. built in 576ms
- Smoke test: 4/4 PASS

## Self-Check: PASSED

Commits verificados:
- `b804e6d` — en git log (feat(crypto): relajar validador multipath...)
- `53af96d` — en git log (feat(ui): dropzone descriptor en TabCifrar...)

Archivos verificados:
- `crates/core/src/validate.rs` — contiene `require_multipath_wildcard`
- `crates/core/tests/validate.rs` — 7 tests con xpubs reales
- `frontend/src/components/TabCifrar.svelte` — tiene dropzone + botón Limpiar
- `frontend/src/components/TabDescifrar.svelte` — tiene botón Limpiar todo
