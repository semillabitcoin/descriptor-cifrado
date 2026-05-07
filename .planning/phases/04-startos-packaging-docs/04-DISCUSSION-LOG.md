# Phase 4: StartOS Packaging + Docs - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 04-startos-packaging-docs
**Areas discussed:** Image pin strategy, README depth/audience, App icon, Registry deployment, Versioning strategy, Update path / breaking changes, Language scope (README + UI i18n)

---

## Image Pin Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| `:latest` tag móvil | Pull siempre la última imagen — riesgo reproducibilidad | |
| `:vX.Y.Z` semver tag | Pin por tag git, requiere coordinación de tags | |
| `@sha256:digest` | Pin por digest exacto, máxima reproducibilidad, bump manual 1 línea | ✓ |

**User's choice:** "lo que veas mejor" → Claude eligió digest pin.
**Notes:** Razón locked-in: el `.s9pk` debe ser determinista. Bump manual = 1 línea por release. Capturado como D-01.

---

## README Depth & Audience

| Option | Description | Selected |
|--------|-------------|----------|
| Solo holder StartOS | Corto, accesible, qué/cómo | |
| Técnico completo | Cripto specs + BIP + attack model | |
| Capas (TL;DR + técnico) | Quickstart visible + secciones técnicas profundas | ✓ |

**User's choice:** "capas y bien hecho"
**Notes:** Estructura propuesta en 6 secciones (TL;DR / Usage / Threat model / Crypto details / Pitfalls / References). DOC-02 (regla de oro) aparece 2x intencionalmente. Capturado como D-02 + D-03.

**Sub-decisión emergente (idioma):** README en **inglés** (alcance audiencia GitHub público + registry).

---

## App Icon

| Option | Description | Selected |
|--------|-------------|----------|
| A) Llave + candado | Clásico crypto/seguridad, genérico | |
| B) Tres llaves entrelazadas con halo cifrado | Multisig metaphor temático, riesgo cargado en thumbnail | |
| C) Logotipo "BED" textual monospace, paleta Semilla Bitcoin | Coste cero, lee bien en thumbnail, marca consistente | ✓ |
| D) Sobre cifrado con sello Bitcoin | Distintivo (evoca armored backup), pide diseñador | |

**User's choice:** "dejamos el logotipo C"
**Notes:** Generar SVG en plan-phase con preview. Fondo negro `#0c0c0c` o naranja `#f7931a`, texto monospace bold. Si no convence al ver preview → fallback D (defer indefinido). Capturado como D-05.

---

## Registry Deployment

| Option | Description | Selected |
|--------|-------------|----------|
| Solo sideload v1 (.s9pk en GH releases) | Universal, sin dependencias | |
| Registry propio Semilla Bitcoin | Catálogo controlado, audiencia primaria | ✓ (primary) |
| StartOS registry oficial Start9 | Review Start9 indefinido | (deferred) |

**User's choice:** "la mejor opción" → Claude eligió combinación: sideload SIEMPRE + publicar al registry propio Semilla Bitcoin tras S9-04.
**Notes:** Sideload nunca se quita (vía universal). Registry oficial Start9 → milestone futuro. Capturado como D-06 + D-07.

---

## Versioning Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Acoplada (semver sincronizado) | `descriptor-cifrado v0.1.0` ⇔ `bed-startos v0.1.0` | ✓ |
| Independiente | Wrapper y backend versionan por separado | |

**User's choice:** "lo que veas"
**Notes:** Acoplada por simplicidad de mensaje al usuario. Primer release **v0.1.0** (no v0.0.1) por completitud del producto. Flow: tag git en `descriptor-cifrado` → CI publica imagen + digest → bump manifest `bed-startos` → tag git → release `.s9pk`. Capturado como D-08 + D-09.

---

## Update Path / Breaking Changes

| Caso | Política |
|------|----------|
| Bump cosmético/UI/bugfix | Patch semver, preserva `/data/encrypted/`, `.bed` existentes siguen leyéndose |
| Crate `bitcoin-encrypted-backup` rompe formato | NO migration auto — milestone nuevo, archivar versión vieja en CHANGELOG |
| Schema filename `/data/encrypted/` rompe | Script idempotente al startup (no v1) |

**User's choice:** "que propones?"
**Notes:** Doctrina propuesta: zero migration code en v1 (la app no almacena xpub, no puede re-cifrar). Formato `.bed` es contrato externo (interop Liana). Capturado como D-10 + D-11.

---

## Language Scope (sub-discussion emergente)

**User input:** "el readme lo podemos hacer en ingles y la app puede estar tb en ingles a parte de español"

| Sub-decisión | Resolución |
|--------------|-----------|
| README en inglés | ✓ Acepto en Phase 4 (D-02). Términos castellanos solo si citan labels UI actuales. |
| App bilingüe EN+ES | ✗ Scope nuevo — UI Phase 2 cerrada, i18n es nueva capability |

**Opciones presentadas para i18n UI:**
| Option | Description | Selected |
|--------|-------------|----------|
| (a) Phase 5 nueva en milestone v0.0.2 | Extender ROADMAP, añadir post-cierre Phase 4 | ✓ (deferred to phase 5) |
| (b) Phase 4.1 INSERTED | Decimal phase entre 4 y 5 | |
| (c) Backlog v0.0.3 | Milestone separado posterior | |

**User's choice:** "como veas" → Claude eligió **(a) Phase 5 nueva**.
**Notes:** Phase 4 mantiene scope limpio (packaging + docs). Phase 5 (i18n) añadirá al ROADMAP vía `/gsd:add-phase` tras cierre Phase 4. Independiente de Phase 4, depende de Phase 2 (UI base). Capturado en `<deferred>` section.

---

## Claude's Discretion (capturado en CONTEXT.md)

- Estructura de directorios dentro de `bed-startos` — planner decide tras invocar skill `start9-packaging`
- Versión exacta de `@start9labs/start-sdk` — researcher verifica al plan
- `actions.ts` custom (ej. "Reset history") — recomendación: sin actions v1
- Properties / env vars expuestas — recomendación: zero env vars configurables v1
- Nivel de detalle de screenshots README — recomendación: screenshots reales tras S9-04

## Deferred Ideas (capturado en CONTEXT.md)

- Phase 5: i18n EN+ES (en este milestone v0.0.2)
- StartOS registry oficial Start9
- SBOM + cosign signing del `.s9pk`
- Migration scripts auto entre formatos `.bed`
- Endpoint `/api/health` custom
- Custom `actions.ts` en manifest
- Iconos artísticos B/D si logotipo C no convence
- README screenshots reales tras S9-04
- Multi-platform Umbrel (XPLAT-01)
- File Browser integration (FB-01/FB-02)
