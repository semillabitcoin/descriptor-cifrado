# Phase 4 UAT — Real StartOS 0.4.0 Device

## UAT Cycle 1 — FAILED (2026-05-07)

### Failure Summary

**Test:** S9-04 — Install + Healthy + Tor URL + LAN URL + round-trip encrypt/decrypt
**Result:** FAIL — web interface never became accessible
**Root cause:** `bed-server` vinculaba en `127.0.0.1:8080` dentro del contenedor, pero el
proxy de ingreso de StartOS vive en otro network namespace y no puede alcanzar la dirección
loopback del contenedor. El health check `checkPortListening` en `/proc/net/tcp` sí pasaba
(lee el netns del contenedor), pero ninguna petición HTTP llegaba al servidor desde Tor ni LAN.

**Fix applied:** `crates/server/src/main.rs` — `BIND_ADDR` cambiado de `"127.0.0.1:8080"` a
`"0.0.0.0:8080"`. Bumped versión 0.1.0 → 0.1.1 en ambos repos. Ver commit `35fb89a` en
descriptor-cifrado y commit `b592a10` en bed-startos.

**Deviation tracking:** [Rule 1 - Bug] SEC-02 mal interpretado: loopback no es necesario para
seguridad dentro de contenedor StartOS — la seguridad proviene del proxy externo y del layer de
auth de StartOS. Corrección aplicada automáticamente per Deviation Rule 1.

---

## UAT Cycle 2 — PENDIENTE (v0.1.1)

Tested: <YYYY-MM-DD>
Device hostname: <e.g., embassy.local>
StartOS version: <output of /system/info/version on dashboard>
Architecture: <x86_64 o aarch64>
Tested by: <usuario>

**Artefactos para instalar:**
- `bed_x86_64.s9pk` (9.4 MB) — `/home/anon/bed-release-v0.1.1/bed_x86_64.s9pk`
- `bed_aarch64.s9pk` (8.8 MB) — `/home/anon/bed-release-v0.1.1/bed_aarch64.s9pk`
- GitHub Release: https://github.com/semillabitcoin/bed-startos/releases/tag/v0.1.1

**NOTA:** Instalar v0.1.1 SOBRE v0.1.0 existente prueba simultáneamente S9-05 (update path + history preservation).

### UAT-1: Install + Healthy (S9-04) — Ciclo 2

**Comando:**
```bash
# Via start-cli sideload:
start-cli -h <device-host>.local package install -s bed_<arch>.s9pk

# O via StartOS UI:
# Settings → Sideload → upload bed_<arch>.s9pk
```

**Expected:** App aparece en dashboard en 30-60s con status "Running" y health "Healthy".
Los logs deben mostrar "bed-server listening" con addr=0.0.0.0:8080.

**Evidence (pegar output):**
- output de start-cli o screenshot de UI
- Línea del dashboard para BED (status + health)
- Primeras 50 líneas de Logs (buscar "Listening on 0.0.0.0:8080")
- Health check ✓ verde junto a BED

Status: <PASS / FAIL — describir>

### UAT-2: Tor URL + round-trip encrypt/decrypt (S9-04) — Ciclo 2

**Pasos:**
1. Desde StartOS dashboard, copiar la URL Tor .onion de BED.
2. Abrir Tor Browser → pegar URL → la página carga.
3. Tab "Cifrar": pegar un descriptor multisig 2-de-3 real (con `<0;1>/*`).
4. Clic "Cifrar". Verificar que aparecen tres outputs: descarga .bed, bloque armored, QR.
5. Tab "Descifrar": subir el .bed y pegar una xpub cosigner.
6. Clic "Descifrar". El descriptor recuperado DEBE coincidir byte a byte con el original.

**Evidence:**
- Prefijo URL .onion (primeros 8 chars OK para log)
- Screenshot de tres outputs visibles
- Round-trip: descriptor original + descriptor recuperado + diff = vacío

Status: <PASS / FAIL>

### UAT-3: LAN URL + round-trip (S9-04) — Ciclo 2

**Pasos:**
1. Desde StartOS dashboard, copiar la URL LAN (algo como `https://bed.<host>.local`).
2. Abrir en browser normal en la misma LAN.
3. Repetir encrypt+decrypt de UAT-2.

**Evidence:**
- `curl -k -sS -o /dev/null -w '%{http_code}\n' https://bed.<host>.local/` → 200
- Round-trip evidence como en UAT-2

Status: <PASS / FAIL>

### UAT-4: History persistence baseline (S9-05 setup) — Ciclo 2

**Pasos:**
1. En BED UI, activar el toggle de historial.
2. Cifrar el mismo descriptor de UAT-2 con historial ON.
3. Tab Historial → verificar que aparece la entrada .bed.
4. Anotar el filename (e.g. `20260507T120000Z-abc12345.bed`).

**Evidence:**
- Filename de la entrada de historial creada
- Screenshot del tab Historial

Status: <PASS / FAIL>

### UAT-5: Update preserves history (S9-05) — Ciclo 2

**NOTA:** Este UAT es GRATUITO si instalas v0.1.1 sobre v0.1.0 ya instalado.
La instalación de v0.1.1 SOBRE v0.1.0 es el test de actualización S9-05.

**Pasos:**
1. Instalar v0.1.1 sobre v0.1.0 existente (StartOS detecta la actualización — no desinstalar).
2. Tras completar (status vuelve a Running + Healthy), abrir tab Historial.
3. Verificar que la entrada .bed de UAT-4 (de v0.1.0) SIGUE listada.
4. Clic descifrar en esa entrada y verificar que el descriptor original se recupera.

**Evidence:**
- Output del comando de instalación de actualización
- Filename de UAT-4 TODAVÍA visible en Historial tras la actualización
- Descifrado exitoso: descriptor recuperado + diff vs original UAT-4

Status: <PASS / FAIL>

### UAT-6: Threat model README accuracy review (DOC-01) — Ciclo 2

**Pasos:**
1. Leer README.md de descriptor-cifrado §"Threat Model" detenidamente.
2. Leer README.md de bed-startos §"Threat model summary" detenidamente.
3. Confirmar que la regla de oro aparece AL MENOS dos veces en cada uno.
4. Confirmar que lo que BED protege vs lo que NO protege está declarado honestamente.

**Evidence:**
- "Leído y preciso" o lista de imprecisiones a corregir antes del cierre de fase.

Status: <PASS / FAIL>

---

## Summary

UAT Cycle 1: FAIL (bind a loopback — fix aplicado en v0.1.1)
UAT Cycle 2: <PENDIENTE>
All UAT-1..6 pass: <YES / NO — describir fallos>
Phase 4 closure recommendation: <APPROVE / FIX issues + retest>
