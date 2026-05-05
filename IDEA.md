# BED Start9 App — Idea Document

App s9pk para StartOS 0.4.0 que cifra y descifra descriptors de Bitcoin siguiendo el draft BIP "Bitcoin Encrypted Backup" (PR `bitcoin/bips#1951`, autor pythcoiner / Wizardsardine).

## Origen y referencias

- BIP draft: https://github.com/bitcoin/bips/pull/1951
- Hilo Delving Bitcoin: https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607
- Crate Rust + CLI de referencia: https://github.com/pythcoiner/encrypted_backup
- GUI nativa de referencia: https://github.com/pythcoiner/bed
- Implementado por Liana v13 como archivos `.bed`

## Por qué existe

En multisig clásico, cada signer device guarda el descriptor completo (todas las xpubs cosigners). Eso significa que comprometer una sola ubicación basta para que un atacante derive todas las direcciones del wallet y vea el saldo y el historial — la privacidad real es 1-de-N, no M-de-N.

El BIP "Bitcoin Encrypted Backup" cifra el descriptor con las propias xpubs de los cosigners. Cualquier participante (1-de-N) puede descifrarlo en solitario. Esto permite distribuir backups del descriptor con redundancia masiva sin riesgo, **siempre que ninguna ubicación contenga simultáneamente el `.bed` y una xpub del multisig**.

Esta app empaqueta esa funcionalidad en una experiencia local privada para usuarios StartOS, eliminando la necesidad de compilar la CLI Rust de pythcoiner.

## Qué hace

**Cifrado:**
- Usuario pega un descriptor (Bitcoin output descriptor estándar, formato miniscript con derivación obligatoria `<0;1>/*`).
- App devuelve tres formas del cifrado:
  - Archivo binario `.bed` (descargable).
  - Versión texto armored estilo PGP (`-----BEGIN BITCOIN ENCRYPTED BACKUP-----`).
  - QR generado del base64 (descargable como PNG).

**Descifrado simétrico:**
- Usuario sube `.bed` (binario o armored pegado) + xpub (texto o archivo).
- App devuelve descriptor recuperado en claro.
- Soporte hardware wallet **fuera de scope** (USB no llega al contenedor StartOS).

**Modo histórico opt-in:**
- Por defecto la app es ephemeral: cifra, entrega resultado, olvida.
- Si el usuario activa toggle "guardar historial", los `.bed` resultantes se persisten en `/data/encrypted/<timestamp>-<short-id>.bed`.
- **El descriptor en claro NUNCA se persiste**, solo el `.bed` cifrado.
- Usuario puede listar y borrar entradas del historial desde la UI.

## Stack

- **Backend:** Rust + axum + tokio. Importa la crate `bitcoin-encrypted-backup` directamente (no shellear la CLI `beb`).
- **Frontend:** SPA mínima (vanilla JS o Svelte) servida desde el mismo backend. Sin CDN externo, sin telemetría, sin fonts remotas.
- **Persistencia (modo opt-in):** archivos en `/data/encrypted/` + sled o SQLite para metadata.
- **Imagen Docker:** `rust:slim` para build, `distroless/cc` para runtime. Target ~5–10 MB.
- **Acceso:** Tor onion + LAN, no clearnet.

## Modelo de amenazas (incluir en README)

- ✅ Protege `.bed` contra atacante que solo encuentra el archivo.
- ❌ NO protege contra compromiso de StartOS durante el cifrado (el descriptor en claro pasa por memoria del proceso).
- ❌ NO protege contra atacante que ya tiene una xpub del multisig.

La app debe borrar el descriptor en claro de memoria/disco inmediatamente tras cifrar.

## Out of scope para v1

- Integración con la app File Browser de StartOS — se evalúa en v2 una vez verificadas las APIs disponibles en StartOS 0.4.0 (HTTP API filebrowser upstream, volúmenes compartidos, carpeta `user-files` del host).
- Hardware wallet support (`devices` feature de la crate).
- Cross-platform a Umbrel — solo StartOS por ahora.
- Multi-usuario / autenticación — la app está protegida por el propio Tor onion + auth de StartOS.

## Restricciones técnicas conocidas

- El BIP exige descriptors con derivación `<0;1>/*` (sin esto, gastar desde dirección 0 expone la xpub on-chain y rompe el cifrado).
- Compatibilidad miniscript v0.12.x (la crate soporta `miniscript_12_0` y `miniscript_12_3_5` via features).
- Repo en organización `semillabitcoin` (preferencia del usuario).
- Email noreply para git: `55397917+4rkad@users.noreply.github.com`.

## Convenciones del proyecto

- StartOS 0.4.0 — invocar skill `start9-packaging` cuando llegue la fase de empaquetado s9pk.
- Verificar en fuente primaria, no inventar (regla del usuario).
- Probar en StartOS real antes de push (regla del usuario).
- Comunicación en castellano, no argentino.
- GHCR packages: hacer pública tras primer push o el deploy falla.
