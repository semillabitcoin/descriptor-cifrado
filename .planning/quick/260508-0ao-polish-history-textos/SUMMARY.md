---
quick_id: 260508-0ao
slug: polish-history-textos
date: 2026-05-08
status: complete
commits:
  - 0ca0734 feat(260508-0ao-01): TabCifrar help reducido (issue C)
  - b2384c6 feat(260508-0ao-02): generalizar "del multisig" → "del descriptor" (issue D)
local_only: true
---

# Polish history + textos UI — SUMMARY

## Hechos

### Issue B — VALIDADO como no-bug (configuración)

Hot-restart del server con `BED_DATA_DIR=/tmp/bed-uat-data` + UAT E2E con descriptor real (`/tmp/bed-test/desc.txt`):

- POST `/api/encrypt` → HTTP 200, `bed_b64` 816 chars, `armored` 909 chars
- POST `/api/history` (con `bed_b64`) → HTTP 200, id `5e8251a4`
- GET `/api/history` → 2 entries
- 2 archivos `.bed` persistidos en `/tmp/bed-uat-data/`

**Conclusión:** `data_dir()` (en `crates/server/src/state.rs:20`) usa `BED_DATA_DIR` si está set; si no, default `/data/encrypted` (no existe en host de dev) → POST falla → toast amarillo. En s9pk producción StartOS volume `main` cubre `/data/encrypted/` y crea el dir. **No hay bug en el código.** Sin cambios.

### Issue C — Help redundante en TabCifrar (1 línea)

Antes (líneas 183-185):
```html
<p id="descriptor-help" class="help">
  Pega el descriptor con derivación multipath <code>&lt;a;b&gt;/*</code> (típicamente <code>&lt;0;1&gt;/*</code>; Liana recovery puede usar <code>&lt;2;3&gt;/*</code>). Nada se envía a internet. Si pegas un descriptor single-chain (<code>/0/*</code>), te propondremos convertirlo automáticamente a multipath.
</p>
```

Después:
```html
<p id="descriptor-help" class="help">
  Nada se envía a internet. El cifrado y descifrado son locales.
</p>
```

Razón: la información de multipath y conversión single-chain ya viven en (a) el placeholder del textarea, (b) el `ConvertSingleChainModal` que se lanza al detectar `/N/*`, (c) el README/CLAUDE.md. El help text duplicaba sin añadir.

Commit: `0ca0734`.

### Issue D — Generalizar "del multisig" → "del descriptor"

El crate `bitcoin-encrypted-backup` cifra cualquier descriptor: singlesig (`wpkh(...)`), multisig (`wsh(sortedmulti(...))`) y miniscript Liana con recovery (`wsh(or_d(...))`). El copy "xpub del multisig" excluía a usuarios singlesig y miniscript que también son target legítimo.

Cambios:

- `frontend/src/components/CifrarOutputs.svelte:51`
  - antes: `Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub del multisig.`
  - ahora: `Binario cifrado. Distribuye copias en ubicaciones que NO contengan ninguna xpub que pertenezca al descriptor.`

- `frontend/src/components/TabDescifrar.svelte:207`
  - antes: `Cualquier xpub cosigner del multisig sirve. La xpub se borra automáticamente tras descifrar.`
  - ahora: `Cualquier xpub del descriptor sirve. La xpub se borra automáticamente tras descifrar.`

Commit: `b2384c6`.

### Audit threat model — surface al usuario

Búsqueda `grep -r "del multisig"` en docs activos:

| Archivo | Línea | Uso |
|---|---|---|
| `IDEA.md` | 17, 53 | Genesis doc — frozen, no tocar |
| `.planning/PROJECT.md` | 56 | Premisa de seguridad clave (active) |
| `.planning/REQUIREMENTS.md` | 71 | DOC-02 wording (semilla del README de Phase 4) |
| `.planning/phases/02-*/*.md` | varios | Phase 2 historical artifacts — frozen, no tocar |
| `.planning/phases/01-*/01-CONTEXT.md` | 160 | Phase 1 historical — frozen, no tocar |

D-31 (sesión 11) lockeó el wording: *"Ninguna ubicación debe contener simultáneamente el `.bed` y una xpub del multisig."* Surface al usuario antes de tocar PROJECT.md / REQUIREMENTS.md — afecta una decisión locked.

Propuesta de generalización:

> "Ninguna ubicación debe contener simultáneamente el `.bed` y una xpub que pertenezca al descriptor cifrado."

**Pendiente decisión usuario** antes de aplicar.

## Smoke test

- `npm run build` → bundle OK (index 75.59 KB, gzipped 26.78 KB; chunks bbqr lazy 49.87 KB gz). Bundle inicial JS+CSS gz < 32 KB, dentro del budget 50 KB.
- `cargo build --release` → 26.5s, binario refrescado con frontend embebido.
- Hot-restart server con `BED_DATA_DIR=/tmp/bed-uat-data`.
- Verificación texto en bundle servido (`grep` sobre `/assets/index-*.js`):
  - 3/3 strings viejos eliminados ✓
  - 3/3 strings nuevos presentes ✓

## Estado al cierre

7 commits sin push (5 de sesión 15 + 2 de esta task). HEAD `b2384c6`. Server corriendo PID en `/tmp/bed-server.log`.

**Issue A (UR `crypto-output` para QR descifrado)** queda para Quick 2 separada — usuario confirmó toggle 3 formatos: plain (auto cuando armored < 500 chars) / BBQR / UR.
