---
id: 260508-mdu
slug: v020-naming-obligatorio
description: Nombre obligatorio del descriptor embebido en filename + display historial + filename del download
date: 2026-05-08
mode: quick
---

# Quick Task 260508-mdu: v0.2.0 — Naming obligatorio del descriptor

## Decisiones de usuario

1. Filename embebido en disco: `<compact>-<id>-<label>.bed`. El binario `.bed` interno NO cambia (sigue interop con la crate `bitcoin-encrypted-backup`).
2. .bed antiguos sin label: NO migración. Display sigue siendo el filename como antes.
3. Sanitización simple: `[a-zA-Z0-9 _-]`, max 80 chars, trim, caracteres inválidos → `-`.
4. Download desde TabCifrar y desde HistoryEntryDetailModal: filename `<label>.bed` (limpio, sin timestamp).

## Cambios

### Backend (`crates/server/src/routes/history.rs`)

- `PostHistoryRequest`: añadir `label: String` (obligatorio, no Option).
- `PostHistoryResponse` y `HistoryEntry`: añadir `label: Option<String>` — None solo para entradas históricas sin label en filename.
- Función `sanitize_label`: `[a-zA-Z0-9 _-]`, trim, max 80 chars, otros chars → `-`. Si tras sanitizar queda vacío o solo whitespace/dashes → `BadRequest`.
- `make_filename(compact, id, label: Option<&str>)`: con label → `<compact>-<id>-<label>.bed`; sin label → `<compact>-<id>.bed` (back-compat para listing de históricos).
- `parse_filename`: retorna `(timestamp, id, Option<label>)`. Detecta tres formas:
  - `<compact:16>-<id:8>.bed` → label=None (legacy)
  - `<compact:16>-<id:8>-<label>.bed` → label=Some
- `find_file_by_id`: buscar por id (8 hex). Coincidencia: filename empieza con `<compact:16>-<id>` y luego viene `.bed` o `-`. Reusar el match con `parse_filename`.
- Tests añadidos: sanitize cases, parse_filename con/sin label, round-trip.

### Frontend `frontend/src/components/TabCifrar.svelte`

- Añadir `let label = $state('')`.
- Input "Nombre" requerido encima del textarea descriptor.
- Validación en cliente: `label.trim()` no vacío, `label.length <= 80`, charset `^[a-zA-Z0-9 _-]+$`. Si falla → `errorMessage` y `errorVisible`. Botón Cifrar deshabilitado si label vacío.
- Pasar `label: label.trim()` en el body de `POST /api/history`.
- Pasar `label` como prop a `<CifrarOutputs />`.
- Limpiar también `label` en `handleLimpiar`.

### Frontend `frontend/src/components/CifrarOutputs.svelte`

- Aceptar `label` como prop adicional.
- `downloadBed`: `${sanitizeLabelForFilename(label) || 'backup'}.bed`. Sanitizar (replace inválidos por `-`) por seguridad. Sin timestamp.
- `downloadQrPng`: `${sanitizeLabelForFilename(label) || 'backup'}.png`.

### Frontend `frontend/src/components/TabHistorial.svelte`

- Display row: si `entry.label` presente → mostrar `{entry.label}` como nombre principal; mantener filename gris/subtítulo. Si `entry.label` ausente → fallback al filename actual (para entradas legacy sin label).

### Frontend `frontend/src/components/HistoryEntryDetailModal.svelte`

- Aceptar `label` opcional como prop.
- `downloadBed`: si `label` presente → `${sanitizeLabelForFilename(label)}.bed`; else → mantener fallback a `filename` actual.
- `downloadQrPng`: misma lógica con `.png`.
- Subtitle: si `label` → mostrar `label` arriba y filename pequeño abajo; sin label → solo filename como ahora.

### Frontend nuevo helper `frontend/src/lib/labelSanitize.js`

- `sanitizeLabelForFilename(s)`: replace cualquier char fuera de `[a-zA-Z0-9 _-]` por `-`, trim, return. Si queda vacío → `''`.
- Reusable en CifrarOutputs y HistoryEntryDetailModal.

## Validación

- Backend tests Rust pasan (`cargo test -p bed-server`).
- Frontend `npm run build` verde.
- Smoke E2E con `BED_DATA_DIR=/tmp/bed-uat-data` (rm -rf primero):
  - Cifrar con label "Mi multisig 3 de 5" + history ON → `POST /api/history 200` → archivo `<ts>-<id>-Mi multisig 3 de 5.bed` en disco.
  - `GET /api/history` retorna entry con `label: "Mi multisig 3 de 5"`.
  - Cifrar sin label → botón Cifrar deshabilitado, no permite envío.
  - Cifrar con caracteres especiales `Ñ@!` → tras sanitizar termina como `--` o `BadRequest` si queda vacío post-sanitización.

## must_haves

- truths:
  - "Backend rechaza POST /api/history sin label o con label vacío post-sanitización"
  - "Filename en disco embebe label tras id"
  - "GET /api/history retorna label en cada entry (None para legacy sin label)"
  - "Download .bed usa <label>.bed sin timestamp"
  - "TabHistorial muestra label como nombre primario; filename como subtítulo"
  - "Botón Cifrar deshabilitado si label vacío"
  - "Entradas legacy sin label en filename siguen apareciendo en historial con su filename"
- artifacts:
  - "crates/server/src/routes/history.rs"
  - "frontend/src/lib/labelSanitize.js"
  - "frontend/src/components/TabCifrar.svelte"
  - "frontend/src/components/CifrarOutputs.svelte"
  - "frontend/src/components/TabHistorial.svelte"
  - "frontend/src/components/HistoryEntryDetailModal.svelte"
- key_links:
  - "history.rs:81 parse_filename, history.rs:117 make_filename"
  - "TabCifrar.svelte:90 POST /api/history"
  - "CifrarOutputs.svelte:18 downloadBed (filename)"
