---
phase: 01-crypto-core-http-api
plan: 03
type: execute
wave: 2
depends_on: ["01-01"]
files_modified:
  - crates/core/src/lib.rs
  - crates/core/src/validate.rs
  - crates/core/src/zeroize.rs
  - crates/core/src/error.rs
autonomous: true
requirements: [CORE-03, CORE-04, CORE-05]
must_haves:
  truths:
    - "Función require_multipath_0_1() rechaza descriptors sin multipath <0;1>/* en cada key"
    - "Newtype ZeroizingDescriptor existe sin Clone/Display/Debug derivados"
    - "Tests de propiedad cubren: bare xpub, single wildcard /*, multipath <2;3> → todos rechazados"
    - "Fixture válida (wsh sortedmulti 2-of-3 con <0;1>/*) → aceptada"
  artifacts:
    - path: "crates/core/src/validate.rs"
      provides: "Validación BIP <0;1>/* multipath"
      exports: ["require_multipath_0_1"]
    - path: "crates/core/src/zeroize.rs"
      provides: "ZeroizingDescriptor newtype sin trait leaks"
      exports: ["ZeroizingDescriptor"]
    - path: "crates/core/src/error.rs"
      provides: "CoreError enum con MissingMultipathWildcard variant"
      exports: ["CoreError"]
  key_links:
    - from: "crates/core/src/validate.rs"
      to: "bitcoin_encrypted_backup::miniscript::Descriptor"
      via: "for_each_key + DescriptorPublicKey::MultiXPub matching"
      pattern: "for_each_key"
    - from: "crates/core/src/zeroize.rs"
      to: "zeroize::Zeroizing"
      via: "wrapper sobre Zeroizing<String>"
      pattern: "Zeroizing<String>"
---

<objective>
Implementar la capa de validación BIP `<0;1>/*` (CORE-03) y el newtype `ZeroizingDescriptor` (CORE-04) en `crates/core`. Define `CoreError` (D-16, internal-to-core; el server lo mapea después). Tests de propiedad (D-25) cubren las 5 entradas inválidas + 1 válida documentadas en RESEARCH.md §"Property test inputs".

Purpose: Cerrar las dos invariantes de seguridad nacientes — wildcard validation (Pitfall #1) y zeroize at parse boundary (Pitfall #4) — en código testeable sin HTTP.
Output: `crates/core` tiene `validate`, `zeroize`, `error` modules; `cargo test -p bed-core` pasa los property tests.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/phases/01-crypto-core-http-api/01-CONTEXT.md
@.planning/phases/01-crypto-core-http-api/01-RESEARCH.md
@.planning/research/PITFALLS.md
@/tmp/bed-test/encrypted_backup/src/descriptor.rs
@/tmp/bed-test/desc.txt
@/tmp/bed-test/xpub.txt

<interfaces>
From bitcoin_encrypted_backup re-exports (miniscript 12.3.5):
```rust
pub use mscript_12_3_5 as miniscript;
// Use as: bitcoin_encrypted_backup::miniscript::{
//   Descriptor, DescriptorPublicKey, ForEachKey,
//   descriptor::{Wildcard, DescriptorMultiXKey},
//   bitcoin::bip32::{ChildNumber, DerivationPath},
// };
```

Pattern from RESEARCH.md "Wildcard Validation Pattern":
```rust
desc.for_each_key(|k| {
    let ok = match k {
        DescriptorPublicKey::MultiXPub(mx) => {
            mx.wildcard == Wildcard::Unhardened
                && mx.derivation_paths.paths().len() == 2
                && mx.derivation_paths.paths()[0].to_string() == "0"
                && mx.derivation_paths.paths()[1].to_string() == "1"
        }
        _ => false,
    };
    // ...
    true
});
```

OPEN QUESTION (RESEARCH.md §1.1): exact accessor for `derivation_paths`. If `.paths()` does not exist, fallback per RESEARCH.md is `mx.to_string().contains("<0;1>/*")` (canonical form). Try `.paths()` first; on compile error fall back to `Display`-based check.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Crear CoreError + ZeroizingDescriptor newtype</name>
  <files>crates/core/src/error.rs, crates/core/src/zeroize.rs, crates/core/src/lib.rs</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-09, D-10, D-11)
    - .planning/research/PITFALLS.md (Pitfall 4 — stack copies)
    - crates/core/src/lib.rs (estado actual)
  </read_first>
  <behavior>
    - `CoreError::MissingMultipathWildcard` exists with thiserror Display "El descriptor debe incluir derivación <0;1>/* en todas las claves..."
    - `CoreError::DescriptorParse` exists
    - `ZeroizingDescriptor::new(s: String) -> Self` wraps string into Zeroizing<String>
    - `ZeroizingDescriptor::as_str(&self) -> &str` accessor for read-only access
    - `ZeroizingDescriptor` does NOT derive Clone, Display, Debug
    - Compile-test: `let _ = format!("{:?}", zd);` MUST fail to compile
  </behavior>
  <action>
    Crear `crates/core/src/error.rs`:
    ```rust
    //! Errors emitted by bed-core. The server crate maps these to HTTP responses
    //! via its own AppError type (D-16). Internal-to-core only.

    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum CoreError {
        #[error("El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain.")]
        MissingMultipathWildcard,

        #[error("No se pudo parsear el descriptor.")]
        DescriptorParse,

        #[error("La xpub proporcionada no descifra este .bed (no es un cosigner válido).")]
        XpubMismatch,

        #[error("El descriptor cifrado excede capacidad QR ({size} > {max} bytes). Usá el archivo .bed o el armored.")]
        QrTooLarge { size: usize, max: usize },

        #[error("error de codificación armored: {0}")]
        Armored(String),

        #[error("error interno de cifrado")]
        Crypto,
    }

    // Map crate's bitcoin_encrypted_backup::Error → CoreError
    impl From<bitcoin_encrypted_backup::Error> for CoreError {
        fn from(e: bitcoin_encrypted_backup::Error) -> Self {
            use bitcoin_encrypted_backup::Error as E;
            match e {
                E::WrongKey | E::NoKey | E::DescriptorHasNoKeys => CoreError::XpubMismatch,
                E::Descriptor | E::Utf8 => CoreError::DescriptorParse,
                _ => CoreError::Crypto,
            }
        }
    }
    ```

    Crear `crates/core/src/zeroize.rs` (D-11 — newtype SIN Clone/Display/Debug):
    ```rust
    //! ZeroizingDescriptor: opaque newtype around `Zeroizing<String>` for cleartext
    //! descriptor handling. Deliberately does NOT implement Clone, Display, or Debug
    //! to make accidental logging or accidental cheap clone impossible (D-11).
    //!
    //! Pass through pipelines by `&mut` reference (D-10) — never by value — to
    //! avoid stack copies leaving cleartext at earlier stack addresses
    //! (PITFALLS #4).

    use zeroize::Zeroizing;

    /// Cleartext descriptor wrapper. The inner `Zeroizing<String>` zeroizes
    /// its heap allocation on drop. Always pass `&mut` references through
    /// helper functions; never move by value.
    pub struct ZeroizingDescriptor {
        inner: Zeroizing<String>,
    }

    impl ZeroizingDescriptor {
        /// Wrap an owned String. The original buffer is moved into Zeroizing
        /// at this single boundary (per D-10).
        pub fn new(s: String) -> Self {
            Self { inner: Zeroizing::new(s) }
        }

        /// Read-only borrow. Use sparingly — caller must not log or clone the
        /// returned slice. Do not store the &str beyond a single function scope.
        pub fn as_str(&self) -> &str {
            &self.inner
        }

        /// Mutable access for in-place zeroize before drop. Most callers do
        /// not need this — Drop on Zeroizing handles it automatically.
        pub fn zeroize_now(&mut self) {
            use zeroize::Zeroize as _;
            self.inner.zeroize();
        }
    }
    ```

    Actualizar `crates/core/src/lib.rs`:
    ```rust
    //! bed-core — pure Bitcoin Encrypted Backup logic.
    pub use bitcoin_encrypted_backup::miniscript;

    pub mod error;
    pub mod validate;
    pub mod zeroize;

    pub use error::CoreError;
    pub use zeroize::ZeroizingDescriptor;
    ```
    NOTA: el módulo `validate` lo crea Task 2 — debe declararse aquí desde ya para que cargo build pase tras Task 2. Por ahora dejarlo declarado; si Task 1 se ejecuta antes que Task 2, comentar `pub mod validate;` temporalmente o crear `validate.rs` vacío para que compile.

    OPCIÓN segura: crear `crates/core/src/validate.rs` con un stub `pub fn require_multipath_0_1(_: &bitcoin_encrypted_backup::miniscript::Descriptor<bitcoin_encrypted_backup::miniscript::DescriptorPublicKey>) -> Result<(), crate::CoreError> { unimplemented!() }` — Task 2 lo reemplaza con la implementación real. NOTA al executor: el `unimplemented!()` está OK aquí porque NO es path de request (es stub temporal); el lint `unwrap_used` no lo rechaza, pero `panic = "warn"` puede dar aviso — aceptable como warning en estado intermedio.
  </action>
  <verify>
    <automated>cargo build -p bed-core 2>&1 | tail -5 && grep -q 'pub struct ZeroizingDescriptor' crates/core/src/zeroize.rs && ! grep -E 'derive\([^)]*Clone|derive\([^)]*Debug|derive\([^)]*Display' crates/core/src/zeroize.rs</automated>
  </verify>
  <acceptance_criteria>
    - `cargo build -p bed-core` exits 0
    - `crates/core/src/error.rs` contiene literal `MissingMultipathWildcard`
    - `crates/core/src/error.rs` mensaje literal: `"El descriptor debe incluir derivación <0;1>/* en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain."` — copia EXACTA, incluyendo acentos
    - `crates/core/src/zeroize.rs` contiene `pub struct ZeroizingDescriptor`
    - `grep -E '#\[derive\([^)]*(?:Clone|Debug|Display)' crates/core/src/zeroize.rs` retorna 0 matches
    - `crates/core/src/zeroize.rs` contiene `Zeroizing<String>`
    - `crates/core/src/lib.rs` exporta `pub use error::CoreError;` y `pub use zeroize::ZeroizingDescriptor;`
  </acceptance_criteria>
  <done>CoreError + ZeroizingDescriptor compilando; mensaje en castellano EXACTO; sin Clone/Debug/Display en el newtype.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implementar validate::require_multipath_0_1 + property tests (D-25)</name>
  <files>crates/core/src/validate.rs, crates/core/tests/validate.rs, crates/core/tests/fixtures/desc.txt</files>
  <read_first>
    - .planning/phases/01-crypto-core-http-api/01-RESEARCH.md (§"Wildcard Validation Pattern" + §"Property test inputs")
    - .planning/phases/01-crypto-core-http-api/01-CONTEXT.md (D-08, D-25)
    - /tmp/bed-test/encrypted_backup/src/descriptor.rs (verificar accessor `mx.derivation_paths.paths()` o variante)
    - /tmp/bed-test/desc.txt (la fixture válida)
  </read_first>
  <behavior>
    - `require_multipath_0_1` con descriptor `wsh(sortedmulti(2,xpub.../<0;1>/*,xpub.../<0;1>/*,xpub.../<0;1>/*))` (la fixture) → `Ok(())`
    - Bare xpub `wsh(pk(xpub...))` → `Err(MissingMultipathWildcard)`
    - Single wildcard `wsh(pk(xpub.../0/*))` → `Err(MissingMultipathWildcard)`
    - Wrong indices `wsh(pk(xpub.../<2;3>/*))` → `Err(MissingMultipathWildcard)`
    - Mixed `wsh(sortedmulti(2,xpub1.../<0;1>/*,xpub2.../<2;3>/*))` → `Err(MissingMultipathWildcard)` (D-08: ALL keys must be <0;1>)
  </behavior>
  <action>
    Copiar fixture: `cp /tmp/bed-test/desc.txt crates/core/tests/fixtures/desc.txt` (crear directorio `crates/core/tests/fixtures/` si no existe).

    Implementar `crates/core/src/validate.rs` (reemplaza el stub de Task 1):
    ```rust
    //! BIP <0;1>/* multipath wildcard validation (CORE-03, D-08).
    //!
    //! The bitcoin-encrypted-backup crate does NOT enforce this — it accepts
    //! any descriptor with at least one non-NUMS key. This module is the
    //! application-layer guard that rejects descriptors which would expose
    //! the xpub on-chain at first spend.

    use bitcoin_encrypted_backup::miniscript::{
        descriptor::Wildcard, Descriptor, DescriptorPublicKey, ForEachKey,
    };

    use crate::CoreError;

    /// Require every key to be a `MultiXPub` with `derivation_paths == [0, 1]`
    /// and `Wildcard::Unhardened`. Rejects bare xpubs, single wildcards, and
    /// non-`<0;1>` multipath indices.
    pub fn require_multipath_0_1(
        desc: &Descriptor<DescriptorPublicKey>,
    ) -> Result<(), CoreError> {
        let mut all_ok = true;

        desc.for_each_key(|k| {
            let ok = match k {
                DescriptorPublicKey::MultiXPub(mx) => {
                    if mx.wildcard != Wildcard::Unhardened {
                        false
                    } else {
                        // Try the canonical accessor. If miniscript 12.3.5 changed
                        // the API, fall back to Display-based check below.
                        let paths = mx.derivation_paths.paths();
                        paths.len() == 2
                            && paths[0].to_string() == "0"
                            && paths[1].to_string() == "1"
                    }
                }
                _ => false, // Single, XPub (single wildcard or none) → reject
            };
            if !ok {
                all_ok = false;
            }
            true // continue iteration
        });

        if all_ok {
            Ok(())
        } else {
            Err(CoreError::MissingMultipathWildcard)
        }
    }
    ```

    NOTA al executor sobre Open Question §1: si `mx.derivation_paths.paths()` no compila, sustituir el bloque MultiXPub branch por:
    ```rust
    DescriptorPublicKey::MultiXPub(mx) => {
        // Display-based fallback: miniscript serializes multipath canonically as "<0;1>/*"
        let s = mx.to_string();
        s.contains("<0;1>/*")
    }
    ```
    Dejar comentario `// FALLBACK: paths() accessor not available; using Display canonical form`.

    Crear `crates/core/tests/validate.rs` con los 5 inputs documentados en RESEARCH.md §"Property test inputs":
    ```rust
    use std::str::FromStr;

    use bed_core::miniscript::{Descriptor, DescriptorPublicKey};
    use bed_core::validate::require_multipath_0_1;
    use bed_core::CoreError;

    fn parse(s: &str) -> Descriptor<DescriptorPublicKey> {
        Descriptor::<DescriptorPublicKey>::from_str(s)
            .unwrap_or_else(|e| panic!("fixture parse failed: {s} -> {e}"))
    }

    const VALID_FIXTURE: &str = include_str!("fixtures/desc.txt");

    // Re-usable raw xpubs from the BIP test vectors (lifted from miniscript test fixtures).
    // These are PUBLIC test xpubs — non-secret.
    const XPUB_A: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
    const XPUB_B: &str = "xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz";

    #[test]
    fn rejects_bare_xpub() {
        let d = parse(&format!("wsh(pk({XPUB_A}))"));
        assert!(matches!(require_multipath_0_1(&d), Err(CoreError::MissingMultipathWildcard)));
    }

    #[test]
    fn rejects_single_wildcard() {
        let d = parse(&format!("wsh(pk({XPUB_A}/0/*))"));
        assert!(matches!(require_multipath_0_1(&d), Err(CoreError::MissingMultipathWildcard)));
    }

    #[test]
    fn rejects_wrong_multipath_indices() {
        let d = parse(&format!("wsh(pk({XPUB_A}/<2;3>/*))"));
        assert!(matches!(require_multipath_0_1(&d), Err(CoreError::MissingMultipathWildcard)));
    }

    #[test]
    fn rejects_mixed_one_good_one_bad() {
        let d = parse(&format!(
            "wsh(sortedmulti(2,{XPUB_A}/<0;1>/*,{XPUB_B}/<2;3>/*))"
        ));
        assert!(matches!(require_multipath_0_1(&d), Err(CoreError::MissingMultipathWildcard)));
    }

    #[test]
    fn accepts_valid_fixture() {
        let d = parse(VALID_FIXTURE.trim());
        assert!(require_multipath_0_1(&d).is_ok(), "valid fixture should pass");
    }

    #[test]
    fn accepts_synthetic_2_of_3_multipath() {
        // Triple-key sortedmulti, all <0;1>/*
        let d = parse(&format!(
            "wsh(sortedmulti(2,{XPUB_A}/<0;1>/*,{XPUB_B}/<0;1>/*,{XPUB_A}/<0;1>/*))"
        ));
        assert!(require_multipath_0_1(&d).is_ok());
    }
    ```

    NOTA: `bed_core::validate` debe estar en el path público — añadir `pub use validate::require_multipath_0_1;` o `pub mod validate;` en `lib.rs` si Task 1 no lo hizo. Verificar `lib.rs` y ajustar.

    Si `XPUB_A`/`XPUB_B` literales no parsean por checksum/length, sustituir por el xpub real de `/tmp/bed-test/key1.txt` (derivar el xpub vía `cat /tmp/bed-test/xpub.txt`).
  </action>
  <verify>
    <automated>cargo test -p bed-core --test validate 2>&1 | tail -15</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p bed-core --test validate` exits 0
    - 6 tests pass: `rejects_bare_xpub`, `rejects_single_wildcard`, `rejects_wrong_multipath_indices`, `rejects_mixed_one_good_one_bad`, `accepts_valid_fixture`, `accepts_synthetic_2_of_3_multipath`
    - `crates/core/src/validate.rs` contiene literal `for_each_key`
    - `crates/core/src/validate.rs` contiene literal `Wildcard::Unhardened`
    - `crates/core/tests/fixtures/desc.txt` existe y es idéntico a `/tmp/bed-test/desc.txt`: `diff crates/core/tests/fixtures/desc.txt /tmp/bed-test/desc.txt` exits 0
    - `cargo clippy -p bed-core --all-targets -- -D warnings` exits 0 (sin unwrap_used violations en `validate.rs`; los `unwrap_or_else` en tests están en `tests/` que NO está bajo el lint del server, pero workspace lints aplican — usar `parse()` helper que ya tiene `unwrap_or_else` permitido)
  </acceptance_criteria>
  <done>Validación BIP wildcard funcional con tests de propiedad cubriendo las 5 entradas inválidas + fixture real válida.</done>
</task>

</tasks>

<verification>
- `cargo build -p bed-core` exits 0
- `cargo test -p bed-core` exits 0 (todos los tests pasan)
- `cargo clippy -p bed-core --all-targets -- -D warnings` exits 0
- `ZeroizingDescriptor` no expone Clone/Display/Debug
- Mensaje de error en castellano EXACTO
</verification>

<success_criteria>
- CORE-03: validation rechaza descriptors sin `<0;1>/*` en cualquier key
- CORE-04: ZeroizingDescriptor newtype existe y previene logging accidental
- CORE-05 (parcial): no `unwrap()`/`expect()` en `crates/core/src/` (lint enforces)
- Property test cubre las 5 entradas de RESEARCH.md
</success_criteria>

<output>
Tras completar, crear `.planning/phases/01-crypto-core-http-api/01-03-core-validate-zeroize-SUMMARY.md` documentando:
- Módulos creados: error, validate, zeroize
- Si se usó `paths()` accessor o el fallback `Display`-based
- Resultado de los 6 property tests
- Hash sha256sum del fixture copiado vs `/tmp/bed-test/desc.txt`
</output>
