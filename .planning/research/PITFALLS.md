# Pitfalls Research

**Domain:** Bitcoin descriptor encryption app on StartOS 0.4.0 (Rust + axum, draft BIP "Bitcoin Encrypted Backup")
**Researched:** 2026-05-05
**Confidence:** HIGH for packaging and memory-safety pitfalls; MEDIUM for BIP format specifics (spec is still a draft)

---

## Critical Pitfalls

### Pitfall 1: Descriptor Without `<0;1>/*` Accepted — Silent Privacy Break

**What goes wrong:**
The encrypt handler accepts a descriptor that lacks a trailing wildcard derivation path (e.g., a bare xpub or a path ending without `/<0;1>/*`). The encryption succeeds without error. The user stores and distributes the `.bed`. Later, when a UTXO received to address index 0 is spent, the xpub is observable on-chain. An observer can then derive all other addresses from the same key — breaking the entire privacy model the app promises to deliver.

**Why it happens:**
The `bitcoin-encrypted-backup` crate delegates descriptor parsing to the `miniscript` crate via `Descriptor::from_str(...)`. Standard miniscript parsing does NOT enforce the trailing derivation path as a security requirement — it is structurally valid to omit it. The BIP draft adds this constraint as a semantic rule that the *application* must enforce, not the library. Developers assume the library handles validation.

**How to avoid:**
Add an explicit validation step in the encrypt handler *before* calling into the crate:

```rust
// Pseudocode — reject any descriptor key that lacks a wildcard derivation
fn validate_descriptor_has_wildcard(desc: &Descriptor<DescriptorPublicKey>) -> Result<(), AppError> {
    for key in desc.iter_pk() {
        match key.wildcard() {
            Wildcard::None => return Err(AppError::MissingWildcard),
            _ => {}
        }
    }
    Ok(())
}
```

Return a clear UI error: "El descriptor debe incluir derivación `<0;1>/*` en todas las claves. Sin esta derivación, gastar desde la primera dirección expone tu xpub on-chain y compromete la privacidad."

Write a property-based test that feeds descriptors without wildcard to the encrypt endpoint and asserts a 400 response.

**Warning signs:**
- The encrypt handler does not have a test case for a bare-xpub descriptor.
- The UI error messages do not mention `<0;1>/*` by name.
- CI passes a descriptor like `wsh(pk(xpub.../0/*))`  (Unhardened but not multipath) without failure.

**Phase to address:** Core encryption logic phase (Phase 1 / the encrypt handler milestone).

---

### Pitfall 2: Cleartext Descriptor Leaked via Tracing Logs

**What goes wrong:**
The axum handler is decorated with `#[tracing::instrument]` or a `TraceLayer` middleware logs request bodies. The descriptor string appears verbatim in stdout/container logs. On StartOS, service logs can be viewed by anyone with admin access, and may persist to disk.

**Why it happens:**
Developers reach for `#[tracing::instrument]` on every handler for observability, which by default captures all function arguments. `TraceLayer::new_for_http()` captures path, method, and headers — an Authorization header or a body-logging extension added later easily pulls in the descriptor.

**How to avoid:**
- Always use `#[tracing::instrument(skip_all)]` on encrypt/decrypt handlers. Log only non-sensitive fields explicitly.
- Never add body-logging middleware to the encrypt or decrypt routes.
- In CI, run a test that sends a known descriptor to the encrypt endpoint and asserts the descriptor string does NOT appear in captured tracing output.
- Add a code review checklist item: "No handler that touches descriptor or xpub input logs the raw value."

**Warning signs:**
- `#[tracing::instrument]` on handler functions without `skip_all`.
- A global body-logging layer applied to all routes.
- Log output visible in development that shows descriptor content.

**Phase to address:** Core encryption logic phase. Harden before any real device testing.

---

### Pitfall 3: Cleartext Descriptor Leaked via Panic Backtrace

**What goes wrong:**
A `unwrap()` or `expect()` inside the encrypt handler panics on malformed input. The Rust default panic handler prints a backtrace to stderr that may include the descriptor string from a stack frame's local variable. This appears in container logs.

**Why it happens:**
Developers use `unwrap()` for ergonomics during early development and forget to harden before deployment. Backtraces in release builds are normally disabled, but enabling `RUST_BACKTRACE=1` for debugging — or using a panic hook that always emits backtraces — can expose data.

**How to avoid:**
- Replace all `unwrap()`/`expect()` in the request path with `?` propagating to an axum error handler.
- Install a custom panic hook with `std::panic::set_hook` that emits a generic "internal error" log line with no backtrace. Do this at startup, not only in debug builds.
- Never set `RUST_BACKTRACE=1` in the container environment in the Dockerfile or StartOS manifest.
- Add a test: send an intentionally malformed payload; assert the response is a structured JSON error, not a 500 with descriptor text.

**Warning signs:**
- `unwrap()` calls in handler functions or the validate/encrypt pipeline.
- `RUST_BACKTRACE` set in Dockerfile `ENV`.
- No custom panic hook registered at startup.

**Phase to address:** Core encryption logic phase. Must be addressed before any user-facing testing.

---

### Pitfall 4: Stack Copies of Descriptor Leave Zeroize Ineffective

**What goes wrong:**
The descriptor is stored in a `Zeroizing<String>` and `.zeroize()` is called after encrypt. But the `String` value was moved through multiple function call boundaries before that point. Each Rust move of a stack-allocated value physically copies the bytes to a new memory address. Only the final address gets zeroed. Earlier copies persist in memory until overwritten by the OS.

**Why it happens:**
This is a well-documented but non-obvious Rust semantics issue. `zeroize` works correctly on the heap allocation backing a `String`, but Rust moves of the `String` struct itself (24 bytes: pointer + length + capacity) happen on the stack. Each move leaves the previous stack slot's bytes intact. More critically, if the descriptor is parsed through intermediate `String` or `str` slices, those intermediate heap allocations may not be zeroized at all.

Documented concretely: a value returned from a function can appear at three distinct memory addresses, with only the final one zeroed (source: https://benma.github.io/2020/10/16/rust-zeroize-move.html).

**How to avoid:**
- Wrap the descriptor in `Zeroizing<String>` at the point of parsing from the HTTP request body, before any function call boundaries.
- Pass it by `&mut` reference through the validation and encrypt pipeline — do not return or move the value.
- Use `Box<[u8]>` for intermediate byte representations where possible, so the heap location is stable.
- Avoid creating any `&str` slice from the descriptor that outlives the `Zeroizing<String>`.
- After encrypt, call `.zeroize()` and then immediately drop.

**Warning signs:**
- Descriptor passed by value (moved) between helper functions.
- `Zeroizing` wrapper applied only at the final step, not at the point of ingestion.
- Intermediate `String::from(descriptor_str)` allocations in helper functions.

**Phase to address:** Core encryption logic phase. Security audit checklist item before any beta release.

---

### Pitfall 5: Persisting Cleartext Descriptor by Mistake

**What goes wrong:**
The history-mode feature writes the `.bed` to `/data/encrypted/`. A developer adds error-recovery logging, debug tracing, or an audit trail that also writes the original descriptor alongside the `.bed`. Or: an exception handler writes the state struct (which contains the descriptor) to a temp file. The cleartext is now on disk — violating the core security invariant.

**Why it happens:**
The history mode involves a file write code path. Any developer touching that path later may add supplementary data to the file write without realizing the descriptor must not appear there. The invariant is implicit rather than enforced structurally.

**How to avoid:**
- Make the invariant structural: the type written to `/data/encrypted/` is `EncryptedBlob` — a type that contains only the ciphertext bytes. It must not implement `From<Descriptor>` or any conversion that carries cleartext.
- Add a CI test that runs the encrypt+save flow and then `grep`s the written file for the descriptor string — asserting zero matches.
- In code review: any `fs::write` call not writing `EncryptedBlob` bytes is a red flag.

**Warning signs:**
- History mode writes a struct that has a `descriptor` field.
- No CI test verifying the file contents of a saved `.bed`.
- `serde_json::to_string` of a state object being written to disk.

**Phase to address:** History mode phase (after core encrypt/decrypt). Add the CI test before enabling the toggle in the UI.

---

### Pitfall 6: File Browser Integration Assumed Available in v1

**What goes wrong:**
A developer designs the `/data/encrypted/` listing or download flow around the assumption that the StartOS File Browser app is available, or that shared volume APIs exist in StartOS 0.4.0. The feature ships, then breaks or never works because the File Browser APIs in 0.4.0 have not been verified against this use case.

**Why it happens:**
File Browser integration is attractive and seems straightforward — it's an existing StartOS app. But the HTTP API of the upstream filebrowser project, the shared volume mounting semantics, and the `user-files` directory behavior in StartOS 0.4.0 have not been verified as stable or accessible to third-party services.

**How to avoid:**
- This is an explicit project constraint (see `PROJECT.md` and `IDEA.md`): File Browser integration is deferred to v2 pending API verification.
- The v1 download flow uses direct browser download via the axum response (Content-Disposition: attachment). This works without any StartOS-specific API.
- Any ticket or task that proposes adding File Browser integration in v1 must be rejected until the APIs are verified on a real StartOS 0.4.0 device.

**Warning signs:**
- Any code referencing `filebrowser`, `user-files`, or shared volume paths to the File Browser container.
- API calls to a `filebrowser` service endpoint from within the app.

**Phase to address:** History mode / download UX phase. The constraint must be documented in that phase's scope.

---

### Pitfall 7: GHCR Package Left Private After First Push — Deploy Fails Silently

**What goes wrong:**
The Docker image is pushed to `ghcr.io/semillabitcoin/bed` during CI. The package is created as private by default (GHCR inherits repository visibility at creation time if not explicitly set). When StartOS attempts to pull the image during installation, it gets a 403/401 and the installation fails. The error message shown to the user may be generic ("installation failed") with no obvious connection to image visibility.

**Why it happens:**
GHCR creates packages as private when the source repository is private, or when no explicit visibility policy is set. The push succeeds from CI (authenticated), but unauthenticated pull by the end-user's StartOS device fails. This matches the user's documented experience: `feedback_ghcr_private_default.md` and `Umbrel preserva app-data — feedback_umbrel_app_data_preservation.md` both confirm this pattern.

**How to avoid:**
- After the first push, immediately set the package to public: GitHub → your org → Packages → bed → Settings → Change visibility → Public.
- Add a step in the CI pipeline that uses `gh` CLI or the GitHub API to set package visibility to public after push.
- Document in the Makefile: "After first build, run `make publish-public` to set GHCR visibility."
- Add to the packaging checklist: verify `curl -I https://ghcr.io/v2/semillabitcoin/bed/manifests/latest` returns 200 (unauthenticated) before testing on a real device.

**Warning signs:**
- No CI step setting package visibility after push.
- First install test done only from a machine authenticated to GHCR (so the private pull succeeds, masking the problem).
- Install failure on a fresh StartOS with a generic error.

**Phase to address:** StartOS packaging phase. Must be the first thing verified after first successful CI push.

---

### Pitfall 8: distroless/cc Runtime Missing Libraries — Surfaces Only on Real Start9

**What goes wrong:**
The app compiles and runs perfectly in local `docker run` tests. On the real StartOS device the container fails to start with a dynamic linker error (`error while loading shared libraries`) or a TLS-related panic. The distroless/cc image provides glibc and libgcc but does NOT include libssl, libcrypto, or a curl/OpenSSL stack. If any dependency (including transitive ones via `reqwest`, `sqlx`, or an HTTPS client) links against the system OpenSSL, startup fails.

**Why it happens:**
Developers test locally using `rust:slim` (which has a full Debian userland) or a standard Debian base. The switch to `distroless/cc` only happens in the final Docker stage. Local `docker run` of the distroless image may pass if the developer machine happens to have the same glibc version. The StartOS ARM device may have a different glibc minor version, or the image may be built for x86_64 only.

**How to avoid:**
- Use `rustls` as the TLS backend for all HTTP clients (axum, reqwest). Disable system OpenSSL entirely: `default-features = false, features = ["rustls-tls"]` for reqwest.
- In the Dockerfile, after the binary is built, run `ldd target/release/bed` and assert no line contains `libssl` or `libcrypto`. Add this as a CI check.
- Build and test the distroless image on ARM (`--platform linux/arm64`) in CI, not just x86_64.
- Test the container startup on a real StartOS device before any release (user preference: `feedback_test_before_push.md`).

**Warning signs:**
- Any `Cargo.toml` dependency with `features = ["native-tls"]` or similar.
- CI only builds `linux/amd64`.
- `ldd` output on the final binary not audited in CI.

**Phase to address:** StartOS packaging phase. Catch before first real-device deploy.

---

### Pitfall 9: Hardware Wallet Path Accidentally Exposed (USB Not in Container)

**What goes wrong:**
A developer enables the `devices` feature flag of the `bitcoin-encrypted-backup` crate to explore hardware wallet decryption. This compiles hardware wallet enumeration code into the binary. At runtime the code tries to enumerate USB HID devices. Inside the StartOS container there are no USB devices; the enumeration panics or returns an error that crashes the decrypt handler for all users.

**Why it happens:**
The `devices` feature is the natural next step for hardware wallet support, and the crate README mentions it. A developer experimenting with future features enables it prematurely. StartOS containers do not have access to the host's USB bus by default.

**How to avoid:**
- `Cargo.toml` must explicitly disable the `devices` feature: `bitcoin-encrypted-backup = { version = "...", default-features = false, features = ["miniscript_latest"] }`.
- This is an explicit out-of-scope item in `PROJECT.md`: "Hardware wallet support — USB no llega al contenedor StartOS 0.4.0."
- Add a CI test that verifies the compiled binary does NOT contain the symbol `enumerate_devices` or any HID-related symbol (using `nm` or `objdump`).

**Warning signs:**
- `features = ["devices"]` appearing in Cargo.toml or Cargo.lock pulling `async-hwi`.
- Any code path in the decrypt handler that calls a device enumeration function.

**Phase to address:** Core encryption logic phase (keep the feature gate locked from the start).

---

### Pitfall 10: Armored Format Header/Encoding Mistakes (Case, BOM, Line Endings)

**What goes wrong:**
The armored output is produced with a subtly wrong header string (e.g., wrong case, trailing space, Windows line endings `\r\n` instead of `\n`), or a BOM (byte-order mark) is prepended by the text serialization layer. Other implementations (the `beb` CLI, Liana v13) reject the armored file during decrypt because the header does not match exactly.

**Why it happens:**
The BIP is a draft. The armored format is modeled on PGP armor but the exact header string, character case, line length, and line ending convention are specified only in the draft text and the reference implementation. Developers often copy the header from memory or from a README that may be stale. Rust `String::from_utf8` can produce a BOM if the source bytes include one from user paste. Windows clipboard may introduce `\r\n`.

**How to avoid:**
- Define the header string as a single `const` in the codebase derived from the reference implementation source, not from memory: `const ARMOR_HEADER: &str = "-----BEGIN BITCOIN ENCRYPTED BACKUP-----";`
- Strip any leading BOM (`\u{FEFF}`) and normalize line endings to `\n` on both input (when parsing armored) and output (when generating armored).
- Write a round-trip test: armored output from this app must be parseable by the `beb` CLI reference implementation. Run this in CI.
- Explicitly test: feed an armored file with `\r\n` line endings to the decrypt endpoint and verify it succeeds.

**Warning signs:**
- Armored header string defined inline in multiple places rather than as a single const.
- No cross-implementation round-trip test in CI.
- The decrypt endpoint returns an error on files generated by Liana v13.

**Phase to address:** Core encryption logic phase — establish the const and round-trip test before the armored output feature ships.

---

### Pitfall 11: Round-Trip Not Tested in CI (Encrypt → Decrypt Must Reproduce Original)

**What goes wrong:**
The encrypt handler and decrypt handler are tested independently in unit tests. The integration is never tested end-to-end. A bug in the key derivation, encoding, or serialization that cancels out within a single path is invisible — until a user encrypts with one version and tries to decrypt with a future version after a dependency update.

**Why it happens:**
Unit tests are faster to write and feel sufficient. Integration tests require spinning up the full axum server and making HTTP calls, which adds complexity developers defer.

**How to avoid:**
- Add a mandatory CI integration test: POST a known descriptor to `/encrypt`, capture the `.bed` bytes, POST them with the correct xpub to `/decrypt`, assert the result equals the original descriptor byte-for-byte.
- Run this test against the reference test vectors from `bitcoin-encrypted-backup/test_vectors/` to catch any crate API misuse.
- Pin the `bitcoin-encrypted-backup` crate version exactly (no `^` semver range) and add a `cargo update --locked` check in CI. Only update after manually re-running the round-trip test.

**Warning signs:**
- No test in CI that exercises both `/encrypt` and `/decrypt` in sequence.
- The `bitcoin-encrypted-backup` dependency uses a permissive version range (`^0.x`).
- No test runs against the crate's own test vectors.

**Phase to address:** Core encryption logic phase. This test should be green before any other feature is added.

---

### Pitfall 12: Tor Onion Misconfigured — Service Binds to 0.0.0.0 Instead of 127.0.0.1

**What goes wrong:**
The axum server binds to `0.0.0.0:PORT`. In a standard Docker setup this exposes the port on all container network interfaces. If the StartOS networking layer or a misconfigured port mapping exposes that interface to the LAN or clearnet, the app (which handles raw descriptor text) is reachable without the Tor protection layer. The app's threat model assumes all traffic goes through the Tor onion or the StartOS-managed LAN proxy.

**Why it happens:**
`0.0.0.0` is the default for most axum/tokio examples and tutorials. Developers copy-paste from docs without considering the container network model. The error is invisible in local testing because the firewall or Docker network isolation masks it.

**How to avoid:**
- Bind explicitly to `127.0.0.1:PORT` in the axum listener: `TcpListener::bind("127.0.0.1:PORT")`.
- The StartOS manifest declares the port and the platform manages exposure. The service itself must not expose on all interfaces.
- Add a startup log line: `"Listening on 127.0.0.1:PORT — not 0.0.0.0"` as a visible assertion.
- In the StartOS packaging phase, verify with a real device that the service is not reachable directly on the LAN IP without going through the StartOS proxy.

**Warning signs:**
- `TcpListener::bind("0.0.0.0:PORT")` in the main.rs or config.
- Service accessible via LAN IP directly in addition to the StartOS-managed URL.

**Phase to address:** StartOS packaging phase. Verify binding address before first real-device deploy.

---

### Pitfall 13: App Update Wipes History — `/data/encrypted/` Not Declared as Persistent Volume

**What goes wrong:**
The user enables history mode and accumulates `.bed` files in `/data/encrypted/`. A new version of the app is installed via StartOS update. The container image is replaced. Because `/data/encrypted/` was not declared as a volume mount in the StartOS manifest, the directory is part of the container filesystem and is discarded when the image is swapped. All history is silently lost.

**Why it happens:**
StartOS container images are immutable and replaced on update. Data persists only if explicitly declared as a volume mount in `manifest.yaml`. Developers familiar with Docker Compose or Kubernetes may assume bind-mount behavior by default, or add the volume declaration to the Dockerfile but not the StartOS manifest. This matches the user's preference: `feedback_umbrel_app_data_preservation.md` — "bakear HTML/configs volátiles en imagen, no bind-mount desde `${APP_DATA_DIR}`."

**How to avoid:**
- In `manifest.yaml`, declare the data volume explicitly:
  ```yaml
  volumes:
    main: /data
  ```
  and ensure `/data/encrypted/` is within that mounted path.
- Write a CI smoke test: start the service, save a `.bed`, simulate an update (replace the image, restart), verify the `.bed` is still present.
- Do NOT rely on writing data to any path inside the container image filesystem (`/app/`, `/tmp/`, etc.). All persistent data must go under the declared volume mount path.

**Warning signs:**
- `/data/encrypted/` is referenced in code but no corresponding volume declared in `manifest.yaml`.
- Encrypted files written to `/tmp/` or a path inside the container image.
- No update-survival test in the packaging test suite.

**Phase to address:** StartOS packaging phase. Volume declaration must be in the manifest before history mode is enabled.

---

### Pitfall 14: Draft BIP Crate Breaking Changes — Silent Format Incompatibility

**What goes wrong:**
A future version of `bitcoin-encrypted-backup` changes the cipher parameters, tag strings, or TYPE encoding (the README explicitly calls these out as triggers for test vector regeneration). The project bumps the crate version. New `.bed` files are produced in the new format. Existing `.bed` files stored by users cannot be decrypted by the new version. Users lose access to their backups silently (the decrypt handler returns an error that looks like "wrong key" rather than "incompatible format version").

**Why it happens:**
The BIP is a draft. Breaking changes are expected. The crate has no documented migration path or format versioning. Semantic versioning (`^0.x`) allows minor bumps that may include format-breaking changes when the spec changes.

**How to avoid:**
- Pin the exact crate version in `Cargo.toml`: `bitcoin-encrypted-backup = "=0.X.Y"`.
- Never allow automatic `cargo update` in CI without a human review step.
- Before any crate version bump: run the full round-trip test against `.bed` files generated by the previous version and assert they are still decryptable. If they are not, document the format break prominently in the release notes.
- Implement a format version check: if the crate exposes version metadata in the encrypted blob, surface it in the decrypt error ("este archivo usa formato v1, la app usa formato v2 — usa la version anterior para descifrar").

**Warning signs:**
- `bitcoin-encrypted-backup = "^0.X"` in Cargo.toml.
- `cargo update` included as a step in CI without a version-pinned lockfile check.
- No test that decrypts a `.bed` produced by a previous crate version.

**Phase to address:** Dependency management — establish pinning in Phase 1 and enforce it throughout.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `unwrap()` in request handlers | Fast prototyping | Panic on bad input exposes stack frames in logs | Never in handlers — use `?` from day one |
| `#[tracing::instrument]` without `skip_all` | Easy observability | Descriptor leaks in logs | Never on encrypt/decrypt handlers |
| `0.0.0.0` bind address | One less config decision | Port exposed on all interfaces | Never — bind to `127.0.0.1` always |
| `^0.x` semver range for draft crate | Automatic security patches | Silent format-breaking update breaks user backups | Never for a draft-BIP crate with no format stability guarantee |
| Skipping real-device test, using local Docker only | Faster iteration | distroless + ARM issues invisible until deploy | Never before a release candidate |
| Zeroize applied at final step only | Simpler code structure | Earlier stack copies of descriptor persist in memory | Never — apply at ingestion point |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `bitcoin-encrypted-backup` crate | Use without checking `<0;1>/*` requirement — assume the crate validates | Add explicit wildcard check in the handler before calling crate API |
| GHCR | Push image, assume it is publicly pullable | Set package visibility to public immediately after first push; verify with unauthenticated curl |
| distroless/cc | Add a crate dependency that links OpenSSL, test only on Debian base | Audit `ldd` output in CI; use rustls for all TLS |
| StartOS manifest volumes | Declare volume in Dockerfile VOLUME directive but not in manifest.yaml | Only the manifest.yaml declaration creates a persistent mount on StartOS |
| axum TraceLayer | Apply globally including encrypt/decrypt routes | Exclude sensitive routes from body/header logging; use `skip_all` on instrumented handlers |
| Tor via StartOS | Bind service to `0.0.0.0`, assume StartOS firewall protects it | Bind to `127.0.0.1`; StartOS manages external exposure via its networking stack |

---

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Accepting descriptor without `<0;1>/*` | Silent privacy break: xpub observable on-chain after first spend | Validate wildcard in handler before encrypt; reject with clear error |
| Logging descriptor or xpub in tracing spans | Descriptor visible in StartOS service logs to admin | `skip_all` on all sensitive handlers; no body-logging middleware on sensitive routes |
| Panic backtrace including descriptor locals | Descriptor visible in logs on malformed input | Custom panic hook with no backtrace; replace all `unwrap()` in request path |
| Stack copies of descriptor not zeroed | Earlier copies of descriptor persist in memory | Pass descriptor by `&mut` reference; wrap in `Zeroizing` at ingestion |
| Any cleartext descriptor written to disk | Violates core security invariant of the app | Type system enforcement: only `EncryptedBlob` can be written to files |
| Service reachable on `0.0.0.0` | Descriptor processing reachable outside Tor/StartOS protection | Bind to `127.0.0.1` only |

---

## "Looks Done But Isn't" Checklist

- [ ] **Descriptor validation:** Encrypt handler appears to work — verify it rejects descriptors without `<0;1>/*` with a test.
- [ ] **Zeroize:** `Zeroizing<String>` is used — verify it is applied at the ingestion point, not after multiple moves.
- [ ] **History mode persistence:** `.bed` files appear in `/data/encrypted/` during testing — verify they survive a container restart by checking the volume declaration in `manifest.yaml`.
- [ ] **GHCR visibility:** CI push succeeds — verify with `curl -I https://ghcr.io/v2/semillabitcoin/bed/manifests/latest` without authentication credentials.
- [ ] **distroless libs:** App starts locally — verify `ldd` output on the release binary contains no `libssl`.
- [ ] **Round-trip:** Encrypt and decrypt work independently — verify they work end-to-end with a CI integration test.
- [ ] **Tor binding:** App is accessible via onion address — verify the service is NOT directly reachable on LAN IP without StartOS proxy.
- [ ] **Armored format:** Armored output renders correctly in the UI — verify the `beb` CLI or Liana v13 can parse and decrypt the armored `.bed`.
- [ ] **Crate version pin:** App builds with latest crate — verify `Cargo.toml` uses `=0.X.Y` exact pin, not `^0.X`.
- [ ] **Hardware wallet feature off:** App compiles — verify `nm` on binary shows no HID/device enumeration symbols.

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Descriptor without `<0;1>/*` accepted | LOW (before users) / HIGH (after distribution) | Add validation and reject retroactively; warn users who may have distributed non-compliant `.bed` files |
| Cleartext in logs | MEDIUM | Rotate logs; audit all log storage; add `skip_all` and redeploy |
| GHCR left private | LOW | Set package to public in GitHub UI; reinstruct users to retry install |
| distroless missing libs | LOW (if caught in dev) / MEDIUM (if in release) | Switch to rustls; rebuild and push new image |
| Volume not declared — history wiped | HIGH (data loss) | Add volume to manifest; data from previous installs is unrecoverable; note in release |
| Crate version bump breaks format | HIGH | Pin to previous version; add migration doc; do not bump until backward compat verified |
| Round-trip broken | MEDIUM | Users with existing `.bed` may be unable to decrypt; requires coordinated version rollback |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Descriptor without `<0;1>/*` accepted | Phase 1: Core encrypt handler | Test: POST descriptor without wildcard → assert 400 response |
| Cleartext in tracing logs | Phase 1: Core encrypt handler | Test: send descriptor, assert string not in captured tracing output |
| Panic backtrace leaks descriptor | Phase 1: Core encrypt handler | Test: send malformed payload, assert response is structured JSON error |
| Zeroize stack copy problem | Phase 1: Core encrypt handler | Code review: confirm `Zeroizing` applied at ingestion, descriptor passed by `&mut ref` |
| Cleartext persisted to disk | Phase 2: History mode | CI test: grep saved `.bed` file for descriptor string, assert zero hits |
| File Browser assumed available | Phase 2: History mode | Scope gate: reject any task referencing File Browser in v1 |
| GHCR left private | Phase 3: StartOS packaging | Verify: unauthenticated `curl` returns 200 for image manifest |
| distroless missing libs | Phase 3: StartOS packaging | CI check: `ldd` output on release binary; real-device test |
| Hardware wallet feature enabled | Phase 1: Core encrypt handler | `Cargo.toml` audit: no `devices` feature; `nm` check in CI |
| Armored format header wrong | Phase 1: Core encrypt handler | CI test: armored output parsed by `beb` CLI reference implementation |
| Round-trip not tested | Phase 1: Core encrypt handler | Mandatory CI integration test before any other feature merges |
| Tor binding on 0.0.0.0 | Phase 3: StartOS packaging | Real-device test: confirm LAN IP not directly reachable |
| Volume not declared — history wiped | Phase 3: StartOS packaging | Smoke test: save `.bed`, simulate update, verify file persists |
| Draft BIP crate breaking change | Phase 1 onward (ongoing) | Exact version pin in Cargo.toml; backcompat test on every crate bump |

---

## Sources

- BIP draft PR discussion — descriptor derivation path requirement: https://github.com/bitcoin/bips/pull/1951
- Delving Bitcoin thread — payload format discussion: https://delvingbitcoin.org/t/a-simple-backup-scheme-for-wallet-accounts/1607
- `bitcoin-encrypted-backup` crate README — test vector regeneration triggers: https://github.com/pythcoiner/encrypted_backup
- Rust zeroize crate documentation: https://docs.rs/zeroize/latest/zeroize/
- Stack copy pitfall with Rust zeroize: https://benma.github.io/2020/10/16/rust-zeroize-move.html
- distroless TLS failure diagnosis: https://lucabaggi.com/posts/ssl-docker/
- distroless base image variants: https://github.com/GoogleContainerTools/distroless/blob/main/base/README.md
- Axum tracing and PII leak: https://github.com/tokio-rs/axum/discussions/1798
- StartOS 0.4.0 volume/container model: https://docs.start9.com/start-os/0.4.0.x/update-040.html
- StartOS service pipeline packaging: https://github.com/Start9Labs/service-pipeline
- GHCR visibility and permission denied errors: https://www.gecko.security/blog/ghcr-github-container-registry-guide
- User preference — real device before push: `feedback_test_before_push.md`
- User preference — GHCR public after first push: `feedback_ghcr_private_default.md`
- User preference — bake volatile data into image: `feedback_umbrel_app_data_preservation.md`

---
*Pitfalls research for: Bitcoin descriptor encryption app on StartOS 0.4.0*
*Researched: 2026-05-05*
