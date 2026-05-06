---
phase: 260506-rx9
plan: 01
subsystem: frontend/xpub-validation
tags: [xpub, liana, interop, normalization, descriptor-style]
dependency_graph:
  requires: []
  provides: [normalizeXpub, xpub-descriptor-style-support]
  affects: [frontend/src/lib/xpub.js, frontend/src/components/TabDescifrar.svelte]
tech_stack:
  added: []
  patterns: [strip-before-validate, normalize-then-test]
key_files:
  created: []
  modified:
    - frontend/src/lib/xpub.js
    - frontend/src/components/TabDescifrar.svelte
decisions:
  - D-XPUB-NORM: strip de derivacion usa /.*$ (primer / hasta final); xpub base58 nunca contiene /, cubre todas las variantes BIP-380 y multipath sin regexes fragiles
metrics:
  duration: ~10 min
  completed: 2026-05-06
---

# Quick Task 260506-rx9: Fix xpub Interop con Liana

**One-liner:** normalizeXpub strippea prefix [fp/path] y sufijo /derivacion para aceptar xpubs descriptor-style de Liana antes del POST /api/decrypt.

## Cambios en xpub.js

**Funcion nueva: `normalizeXpub(text)`**

Recibe cualquier string xpub (bare o descriptor-style) y devuelve la xpub bare:

1. Si no es string, devuelve `''`.
2. Trimea whitespace.
3. Strip prefix: `s.replace(/^\[[^\]]*\]/, '')` — elimina `[fingerprint/path]` si presente.
4. Strip sufijo: `s.replace(/\/[^\s]*$/, '')` — elimina todo desde el primer `/` hasta el final.
5. Devuelve el resultado.

**Regex final usada:** dos replaces secuenciales; `XPUB_REGEX` sin cambios.

**Modificacion en `validateXpub(text)`:**
- Antes: testaba `XPUB_REGEX` contra `text.trim()`.
- Ahora: llama `normalizeXpub(text)` primero, luego testa `XPUB_REGEX` contra el resultado.

## Decision D-XPUB-NORM — Strip permisivo (locked)

El strip de derivacion usa `/\/[^\s]*$/` (primer `/` hasta el final) en vez de enumerar regex fragiles por variante (`/<0;1>/*`, `/*`, `/0/*`, multipath, etc.).

**Por que es seguro:** una xpub base58 NUNCA contiene `/`. El caracter `/` solo aparece en el sufijo de derivacion BIP-380. Por tanto, eliminar desde el primer `/` es inequivoco y no puede truncar la xpub.

**Cobertura:** `/<0;1>/*`, `/*`, `/0/*`, `/1/*`, multipath `/<m;n>/*`, cualquier variante futura.

## Cambios en TabDescifrar.svelte

- Import: `import { validateXpub, normalizeXpub } from '../lib/xpub.js';`
- POST body: `formData.append('xpub', normalizeXpub(xpubText))` (normalizeXpub ya hace trim)
- Sin cambios en: `xpubReady = $derived(validateXpub(xpubText))` (validateXpub ya normaliza)
- Sin cambios en: limpieza `xpubText = ''` post-descifrado

## Resultado del Smoke Test (automatizado, backend corriendo en 127.0.0.1:8080)

**Test 1 — xpub bare clásica (regresion):**
- Input: `xpub6Euvf9G...TTVMZ` (bare)
- HTTP: 200
- Output: descriptor 2-of-3 sortedmulti correcto

**Test 2 — Bug original confirmado:**
- Input: `[68a9ec24/48h/0h/0h/2h]xpub6Euvf9G...TTVMZ/<0;1>/*` (descriptor-style raw, sin normalizar)
- HTTP: 422 DESCRIPTOR_PARSE (el backend no acepta el formato descriptor; confirma el bug)

**Test frontend via build:**
- `npm run build` completado sin errores, 895 ms, bundle 30 KB gzipped.
- Los 6 casos de validateXpub + normalize-roundtrip pasan con exit 0 (node inline).

**Correspondencia con desc.txt:**
- El descriptor recuperado con xpub bare coincide estructuralmente con `desc.txt`.
- Diferencia esperada: miniscript re-serializa `48h` como `48'` (ambos son BIP-380 validos; decision documentada en STATE.md desde Phase 1).

## Confirmacion: Backend NO modificado

`git diff HEAD src/` = 0 lineas. Contract Phase 1 (/api/decrypt espera xpub bare) preservado sin cambios.

## Deviations from Plan

None — plan ejecutado exactamente como estaba descrito.

## Self-Check

- [x] `frontend/src/lib/xpub.js` exporta `normalizeXpub` y `XPUB_REGEX`; `validateXpub` normaliza antes de testar.
- [x] `frontend/src/components/TabDescifrar.svelte` importa y usa `normalizeXpub`.
- [x] `npm run build` limpio.
- [x] Smoke test HTTP 200 con xpub bare; HTTP 422 con descriptor-style raw (bug original confirmado).
- [x] Backend Rust intacto (0 lineas diff en src/).
- [x] Commits: ced4f08 (Task 1), ec5f761 (Task 2).
