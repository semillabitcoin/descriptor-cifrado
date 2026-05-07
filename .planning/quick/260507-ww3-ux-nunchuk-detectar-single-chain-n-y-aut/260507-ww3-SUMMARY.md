---
phase: quick-260507-ww3
plan: "01"
subsystem: frontend-ux
tags: [single-chain, multipath, nunchuk, descriptor, modal, bip389, bip44]
dependency_graph:
  requires: [frontend/src/lib/xpub.js pattern, Modal.svelte/ConfirmDeleteModal.svelte patterns]
  provides: [detectSingleChain, convertSingleChainToMultipath, ConvertSingleChainModal]
  affects: [TabCifrar.svelte, POST /api/encrypt flow]
tech_stack:
  added: []
  patterns: [standalone modal (backdrop+panel), Svelte 5 $state/$props/$effect, par-impar sibling rule]
key_files:
  created:
    - frontend/src/lib/descriptor.js
    - frontend/src/components/ConvertSingleChainModal.svelte
  modified:
    - frontend/src/components/TabCifrar.svelte
decisions:
  - Modal pattern standalone (ConfirmDeleteModal) en lugar de reutilizar Modal.svelte con snippets — mismo patrón ya establecido en el proyecto, evita incompatibilidades Svelte 5
  - Regla par-impar sibling alineada con Sparrow/drongo OutputDescriptor.java y BIP44/BIP389
  - handleSingleChainConfirm actualiza descriptor (textarea) ANTES de llamar handleCifrar para que el POST use el valor convertido y el textarea refleje lo cifrado
metrics:
  duration: "~10 min"
  completed: "2026-05-07"
  tasks: 3
  files: 3
---

# Quick Task 260507-ww3: UX Nunchuk — Detectar descriptor single-chain y proponer conversión multipath

**One-liner:** Modal de confirmación que detecta descriptores `/N/*` (Nunchuk Desktop) y propone conversión a `<base;base+1>/*` aplicando la regla par-impar sibling de BIP44/BIP389 antes de cifrar.

## Archivos creados / modificados

| Archivo | Estado | Descripción |
|---------|--------|-------------|
| `frontend/src/lib/descriptor.js` | Creado | `detectSingleChain(s)` y `convertSingleChainToMultipath(s)` — helpers puros |
| `frontend/src/components/ConvertSingleChainModal.svelte` | Creado | Modal de confirmación con preview, explicación en castellano y nota BIP44/BIP389 |
| `frontend/src/components/TabCifrar.svelte` | Modificado | Interceptor en `handleCifrar`, wiring del modal, texto de ayuda actualizado |

## Commits

| # | Hash | Mensaje |
|---|------|---------|
| 1 | `8110736` | feat(260507-ww3-01): descriptor.js con detectSingleChain y convertSingleChainToMultipath |
| 2 | `071f5ee` | feat(260507-ww3-02): ConvertSingleChainModal.svelte — modal confirmación single-chain |
| 3 | `a21e6ba` | feat(260507-ww3-03): TabCifrar intercept single-chain con ConvertSingleChainModal |

## Decisiones tomadas

**1. Patrón modal standalone (backdrop+panel) en lugar de Modal.svelte con snippets**

Se optó por el patrón de `ConfirmDeleteModal.svelte` (backdrop propio + panel con role="dialog") porque es el patrón ya establecido en el proyecto y evita potenciales problemas con el slot/snippet de Modal.svelte en Svelte 5. El plan lo indicaba como fallback preferible.

**2. Regla par-impar sibling para conversión**

```
pairFor(n) = [floor(n/2)*2, floor(n/2)*2 + 1]
/0/* y /1/* → <0;1>/*   (receive=0, change=1, BIP44)
/2/* y /3/* → <2;3>/*   (Liana recovery)
```

Alineada con Sparrow Wallet (drongo `OutputDescriptor.java`) y BIP44/BIP389. El caso crítico `/1/* → <0;1>/*` (NO `<1;2>/*`) se verificó con assert explícito.

**3. Actualizar textarea antes de re-llamar handleCifrar**

`handleSingleChainConfirm` asigna `descriptor = singleChainConverted` antes de llamar `handleCifrar()`. Esto garantiza que:
- El POST `/api/encrypt` recibe el descriptor convertido
- El textarea del usuario muestra el valor que realmente se cifró

## Resultado del build

```
Frontend (Vite):
  dist/assets/index-BaZD6EVi.js    75.92 kB │ gzip: 26.86 kB
  dist/assets/browser-BeHB6bsH.js  23.46 kB │ gzip:  8.84 kB
  dist/assets/bbqr-fj_Xli65.js    145.74 kB │ gzip: 49.87 kB
  ✓ built in 748ms

Cargo release:
  Compiling bed-server v0.1.1
  Finished `release` profile [optimized] target(s) in 24.49s
  Binary: target/release/bed-server (3.7 MB)
```

## Smoke test manual (UAT — conducido por el usuario)

### Requisitos

- Servidor en ejecución: `./target/release/bed-server` (en `/home/anon/descriptor-cifrado`)
- Abrir: http://localhost:8080 → pestaña **Cifrar**

### Casos de prueba

**Caso 1: Descriptor receive-only Nunchuk (/0/*) — DEBE activar modal**

1. Pegar en el textarea:
   ```
   wsh(sortedmulti(2,[aabbccdd/48h/0h/0h/2h]xpub6Abc111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111/0/*,[aabbccdd/48h/0h/0h/2h]xpub6Def222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222/0/*))#abc12345
   ```
2. Clic en **Cifrar**
3. Resultado esperado: aparece modal "Descriptor receive-only detectado"
4. El preview muestra el descriptor con `/<0;1>/*` en lugar de `/0/*` y sin `#abc12345`
5. Clic en **Convertir y cifrar** → textarea se actualiza, se lanza el POST, respuesta HTTP 200
6. El textarea refleja el descriptor convertido (sin checksum)

**Caso 2: Descriptor change-only Nunchuk (/1/*) — CRÍTICO, debe producir `<0;1>/*` no `<1;2>/*`**

1. Pegar:
   ```
   wpkh([aabbccdd/84h/0h/0h]xpub6Xyz111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111/1/*)#deadbeef
   ```
2. Clic en **Cifrar**
3. Modal aparece, el preview muestra `/<0;1>/*` (NO `/<1;2>/*`)
4. Confirmar → HTTP 200

**Caso 3: Cancelar no lanza POST**

1. Pegar descriptor con `/0/*` → modal aparece
2. Clic en **Cancelar** → modal desaparece, descriptor sin cambios, sin POST

**Caso 4: Descriptor ya multipath no activa modal**

1. Pegar:
   ```
   wsh(sortedmulti(2,[aabbccdd/48h/0h/0h/2h]xpub6Abc111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111/<0;1>/*,[aabbccdd/48h/0h/0h/2h]xpub6Def222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222/<0;1>/*))#checksum
   ```
2. Clic en **Cifrar** → no aparece modal, va directo a cifrar

**Caso 5: Descriptor Liana recovery chain (/2/*)**

1. Pegar descriptor con `/2/*`
2. Modal aparece, preview muestra `/<2;3>/*`

## Notas

**CAMBIOS LOCALES — NO pusheados a origin**

Todos los commits son locales en la rama `main`. Pendiente push + bump de versión s9pk en sesión separada.

El binario `target/release/bed-server` embebe la SPA actualizada vía rust-embed y está listo para pruebas manuales en localhost:8080.
