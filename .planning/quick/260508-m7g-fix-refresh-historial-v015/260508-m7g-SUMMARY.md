---
id: 260508-m7g
slug: fix-refresh-historial-v015
status: complete
date: 2026-05-08
commit: d4e031f
---

# Quick Task 260508-m7g — SUMMARY

## Goal

Issue v0.1.5-A: tras `POST /api/encrypt` + `POST /api/history` exitosos, la pestaña Historial mostraba lista cacheada de mount inicial; sólo `Ctrl+Shift+R` actualizaba.

## Causa raíz

`App.svelte:38-49` mantiene los tres `<section role="tabpanel">` montados; togglea `hidden={appState.activeTab !== '...'}`. TabHistorial nunca se desmonta → su `$effect(() => { void loadList(); })` corre una sola vez en mount inicial.

## Cambios

| Archivo | Cambio |
|---------|--------|
| `frontend/src/stores/app.svelte.js` | Añadir `historyVersion: 0` al `$state`; export `bumpHistoryVersion()`. |
| `frontend/src/components/TabCifrar.svelte` | Importar `bumpHistoryVersion`; invocarlo tras `postJson('/api/history', ...)` exitoso (no en el catch del warning). |
| `frontend/src/components/TabHistorial.svelte` | Leer `appState.historyVersion` dentro del `$effect` para que la dependencia reactiva fuerce re-fetch en cada cifrado. |

## Validación

- `npm run build` ✅ verde, 587ms, 182 módulos. Bundle `index-*.js` = 75.67 KB (gzip 26.81 KB) — sin regresión vs v0.1.4.
- Warning preexistente `.help code unused selector` no alterado.

## Commit

- `d4e031f fix(260508-m7g): refrescar TabHistorial tras cifrar exitoso (issue v0.1.5-A)` — 3 files, +16/-2.

## Pendientes para release v0.1.5

Cadena estándar idéntica a v0.1.4:
1. `git push origin main`
2. `git tag v0.1.5 && git push origin v0.1.5` → docker.yml CI → capturar nuevo digest GHCR
3. En `bed-startos`: pin nuevo digest + crear `startos/versions/v0.1.5.1.ts` (release notes 5 idiomas) + actualizar VersionGraph `current=v_0_1_5_1`, `other=[v0.1.0.1, v0.1.1.1, v0.1.2.1, v0.1.3.1, v0.1.4.1]` + tag v0.1.5 → release.yml → s9pk
4. UAT en device del usuario.

## Limitaciones / no-cambios

- Borrado de entrada (TabHistorial.handleDelete) ya actualiza la lista localmente; no necesita signal.
- Toast amarillo "Cifrado OK pero no se guardó en historial" (catch) NO bumpea — correcto, no escribió nada.
- Ningún cambio en backend.
