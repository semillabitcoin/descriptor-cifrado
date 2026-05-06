# Phase 2: SPA Frontend + History - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-06
**Phase:** 02-spa-frontend-history
**Areas discussed:** Estilo / branding, Cifrar UX, Descifrar UX, History UX, Threat model UX, Feedback ops, Persistencia client-side, Form Descifrar inputs, QR display, Empty states, Responsive, Accesibilidad, Tipografía, Persistencia history detalle, QR descrifrar formato, Nivel a11y

---

## Áreas grises identificadas (presentación inicial)

10 áreas grises identificadas y presentadas al usuario en formato libre (sin AskUserQuestion porque el usuario pidió ver todas para responder en bloque):

1. Estilo visual / branding
2. Threat model UX (UI-03)
3. Layout & navegación de History
4. Feedback de operaciones
5. Persistencia client-side del toggle history
6. Form Descifrar — diseño de inputs
7. QR display en pantalla
8. Empty states & estados intermedios
9. Responsive / dispositivo objetivo
10. Accesibilidad básica

**Respuesta del usuario (free-form, todas a la vez):**

> "el estilo que sea agradable de ver, con buena tipografia, con modo claro y oscuro. 2. la idea es introducir un descriptor y que lo cifre en forma de archivo, texto y QR y que se de a elegir en que formato se quiere obtener. Luego que se pueda introducir un archivo o descriptor cifrado y que con una clave publica se pueda obetener. La introduccion de un xpub deberia ser efimera, sin guardar. Si se podría guaradr la lista de descriptores cifrados con su archivo y texto cifrado y QR. 3. lo que veas mas adecuado. 4. Tambien. 5.lo que veas. 6. subes un archivo y te sale un cuadro para meter el xpub y lugo te sale QR, texto y archivo con el descriptor sin difrar y ya eliges que formato quieres usar. El QR deberia ser funcionable para Sparrow nunchuk y otras. 7. si, se muestra QR. 8. lo que veas. 9. si que sea reposive para movil. 10. no se, no se sobre ello. guiame"

**Captura de decisiones desde respuesta libre:**

| Área | Decisión |
|---|---|
| Estética | Tipografía cuidada, modo claro y oscuro |
| Cifrar UX | Mostrar los tres formatos, usuario elige |
| Descifrar UX | Sube .bed → input xpub → tres outputs (text/file/QR) del descriptor recuperado |
| xpub | Efímero, no se guarda |
| History contenido | Lista de cifrados con archivo + texto + QR accesibles |
| Threat model UX | Claude's discretion |
| Feedback ops | Claude's discretion |
| Persistencia toggle | Claude's discretion |
| Form Descifrar | Linear: archivo → xpub → resultado |
| QR Descifrar | Compatible Sparrow / Nunchuk / wallets |
| QR on-screen | Sí, renderizado |
| Empty states | Claude's discretion |
| Responsive | Sí, mobile + desktop |
| A11y | Pidió guía |

---

## Tipografía

| Option | Description | Selected |
|--------|-------------|----------|
| Inter + JetBrains Mono | UI Inter (legible, ~30 KB woff2 variable) + descriptors JetBrains Mono. Self-hosted via rust-embed. | ✓ |
| Geist + Geist Mono | Tipografía Vercel (similar a Inter). Sans + Mono. Self-hosted. | |
| Solo system-ui | Cero peso, look inconsistente entre OS. | |

**User's choice:** Inter + JetBrains Mono (recommended)
**Notes:** Razón: combinación estándar 2025/2026, look profesional, JetBrains Mono específica para textos técnicos como descriptors y xpubs (mejor legibilidad de chars `0Ol1` y similares).

---

## History — qué se persiste

| Option | Description | Selected |
|--------|-------------|----------|
| Solo .bed, regenerar armored/QR on-demand | Backend persiste solo .bed binario; armored y QR se regeneran al servir vista/listado. Menos disco, sin redundancia. | ✓ |
| Guardar los tres archivos en disco | `<id>.bed` + `<id>.armored.txt` + `<id>.qr.png`. Más espacio, más superficie de exposure. | |

**User's choice:** Solo .bed, regenerar on-demand (recommended)
**Notes:** Coherente con "armored es solo encoding base64+headers" y "QR es la imagen del armored" — derivables 100%. Menos surface area en disco.

---

## QR del descriptor recuperado (Descifrar)

| Option | Description | Selected |
|--------|-------------|----------|
| QR plano único, advertir si excede ~2900 bytes | Sparrow/Nunchuk leen QR plano de descriptor. Multisig 2-of-3 típico cabe. Si excede, mensaje "usa archivo". | |
| BBQR animado siempre | QR multi-frame Coinkite, soportado por Sparrow + Nunchuk. Cubre cualquier tamaño pero más complejo. | |
| QR plano + caer a BBQR si excede | Lo mejor de ambos: plano cuando cabe, BBQR cuando no. Máxima compatibilidad. | ✓ |

**User's choice:** QR plano + caer a BBQR si excede
**Notes:** Decisión de máxima compatibilidad. BBQR JS lib (npm `bbqr` o equivalente) requiere verificación en research-phase: license, tamaño bundle, mantenimiento. Alternativa: implementar BBQR client-side desde spec si la lib no satisface. Sparrow / Nunchuk QR support detallado debe verificarse en research (no asumir).

---

## Accesibilidad — Nivel

| Option | Description | Selected |
|--------|-------------|----------|
| WCAG AA básico | Labels y ARIA en forms, contraste AA, focus visible, navegación teclado, HTML semántico. Esfuerzo bajo, beneficio alto. | ✓ |
| Mínimo (solo HTML semántico) | Reduce trabajo pero excluye usuarios con lectores de pantalla. | |
| Skip — no es prioridad | Cero esfuerzo. App funcional pero con barreras. | |

**User's choice:** WCAG AA básico (recommended)
**Notes:** Usuario pidió guía explícita. Recomendación aceptada — el esfuerzo extra para a11y básica es mínimo en una app de form simple, y beneficia la base de usuarios StartOS (algunos de los cuales pueden depender de lectores de pantalla por privacidad o accesibilidad).

---

## Claude's Discretion (delegado por el usuario)

Áreas donde el usuario delegó la decisión a Claude:

- **Threat model UX**: formato exacto de display → resuelto con sección colapsable HTML `<details>` + callout destacado del aviso clave.
- **Feedback de operaciones**: spinners, toasts, confirmaciones → resuelto con spinner inline en botón + toast superior derecha + label change para copy + modal de confirmación para borrado.
- **Persistencia client-side toggle**: resuelto con `localStorage` `bed.historyEnabled` (default OFF first-visit), badge visible cuando ON, sin estado backend (eliminado AtomicBool global).
- **Empty states**: texto centrado + icono mínimo, sin SVG ilustración pesada.
- **Layout History**: 3ª pestaña condicional al toggle.
- **Routing**: state-based interno con stores Svelte, sin URL hash.
- **Paleta de colores, spacing scale, animaciones**: Claude's discretion durante UI-SPEC.

---

## Deferred Ideas (notadas durante la discusión)

- **PERS-01 simplificado** — D-19 elimina la necesidad de backend persistence del toggle (era originalmente Phase 2 con AtomicBool, deferido a v1.x con file-models). Ahora el toggle es 100% client-side.
- **BBQR para Cifrar** — fuera de scope; requiere cambio en `/api/encrypt` contract.
- **UX2-01 / UX2-02 / UX2-03 / UX2-04** — todos confirmados deferred a v2 (heredado de Phase 1 deferred section).
- **i18n / multi-language** — defer; estructura de strings centralizada permite añadir locales sin rework.

---

*Discussion captured: 2026-05-06*
