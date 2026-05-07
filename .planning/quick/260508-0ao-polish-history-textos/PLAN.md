---
quick_id: 260508-0ao
slug: polish-history-textos
date: 2026-05-08
description: Validar Issue B (history toast) + fix Issues C y D — help redundante TabCifrar y texto que excluye singlesig/miniscript en CifrarOutputs/TabDescifrar
status: in-progress
---

# Polish history + textos UI (260508-0ao)

## Contexto

Cierre sesión 15 dejó 4 issues pendientes detectados al UAT del s9pk v0.1.2. Esta quick task ataca B (validar), C (1-line fix), D (1-line fix + audit). Issue A (UR `crypto-output` para QR descifrado) es Quick 2 separada — toggle 3 formatos plain/BBQR/UR confirmado por usuario.

## Tareas

### B — Validar issue history toast (no bug, configuración)

**Hipótesis:** server local sin `BED_DATA_DIR` set escribe a `/data/encrypted/` default que no existe en host → POST `/api/history` falla → toast amarillo "Cifrado OK, pero no se guardó en historial" (TabCifrar.svelte:92).

**UAT (con BED_DATA_DIR=/tmp/bed-uat-data):**
- POST /api/encrypt con descriptor real `/tmp/bed-test/desc.txt` → HTTP 200, bed_b64 816 chars ✓
- POST /api/history con bed_b64 → HTTP 200, id `5e8251a4` ✓
- GET /api/history → 2 entries ✓
- `/tmp/bed-uat-data/*.bed` → 2 archivos ✓

**Conclusión:** Issue B = falta de env var en dev, no bug. En s9pk prod el volume `main` cubre `/data/encrypted` (D-15 sesión 11). Fix: documentar mejor en hot-restart command de la memory file. Sin cambios de código.

### C — Quitar help redundante en TabCifrar.svelte

**Antes (líneas 183-185):**
```html
<p id="descriptor-help" class="help">
  Pega el descriptor con derivación multipath <code>&lt;a;b&gt;/*</code> (típicamente <code>&lt;0;1&gt;/*</code>; Liana recovery puede usar <code>&lt;2;3&gt;/*</code>). Nada se envía a internet. Si pegas un descriptor single-chain (<code>/0/*</code>), te propondremos convertirlo automáticamente a multipath.
</p>
```

**Después:** reducir a la garantía de privacidad. El placeholder ya muestra el formato; el modal de single-chain explica la conversión cuando aplica; el README documenta requisitos.

```html
<p id="descriptor-help" class="help">
  Nada se envía a internet. El cifrado y descifrado son locales.
</p>
```

### D — Generalizar texto "del multisig" a "del descriptor"

**`frontend/src/components/CifrarOutputs.svelte:51`:**
- Antes: `Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub del multisig.`
- Después: `Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub que pertenezca al descriptor.`

**`frontend/src/components/TabDescifrar.svelte:207`:**
- Antes: `Cualquier xpub cosigner del multisig sirve. La xpub se borra automáticamente tras descifrar.`
- Después: `Cualquier xpub del descriptor sirve. La xpub se borra automáticamente tras descifrar.`

**Audit threat model (active docs, no historical):**
- `PROJECT.md:56` — premisa de seguridad clave
- `REQUIREMENTS.md:71` — DOC-02 wording (semilla del README de Phase 4)

Ambos repiten la frase "ninguna xpub del multisig". Surfaceo al usuario antes de tocar — afecta D-31 locked (sesión 11).

## Smoke test

1. `(cd frontend && npm run build)` — verificar bundle build OK
2. `cargo build --release` — re-compilar binary embed (no requiere si solo UI)
3. Hot-restart server con `BED_DATA_DIR=/tmp/bed-uat-data`
4. `curl http://127.0.0.1:8080/` HTTP 200
5. Verificar que el HTML servido contiene los textos nuevos (no los viejos)

## Atomic commits

- `feat(ui): TabCifrar help reducido a garantía de privacidad (issue C)`
- `feat(ui): generalizar copy "del multisig" → "del descriptor" en CifrarOutputs y TabDescifrar (issue D)`
- `docs(quick-260508-0ao): SUMMARY + STATE row`

Sin push (decisión sesión 15: cambios solo en local hasta consolidar bump v0.1.x).
